import 'dart:io';

import 'package:flutter/material.dart';

import '../../services/file_dialog_service.dart';
import '../../services/security_bridge.dart';
import '../../shared/drop_input_field.dart';
import '../../shared/page_header.dart';
import '../../shared/scene_switch.dart';
import '../../shared/section_card.dart';

class StegoPage extends StatefulWidget {
  const StegoPage({super.key});

  @override
  State<StegoPage> createState() => _StegoPageState();
}

enum _StegoScene { embed, extract }

class _StegoPageState extends State<StegoPage> {
  final _bridge = const SecurityBridge();
  final _fileDialog = const FileDialogService();
  final _formKey = GlobalKey<FormState>();
  final _carrierController = TextEditingController();
  final _payloadController = TextEditingController();
  final _passwordController = TextEditingController();

  _StegoScene _scene = _StegoScene.embed;
  bool _isSubmitting = false;

  @override
  void dispose() {
    _carrierController.dispose();
    _payloadController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _run() async {
    final carrier = _carrierController.text.trim();
    if (!_formKey.currentState!.validate() || !_validateInputs()) {
      return;
    }

    final saveDirectory = await _fileDialog.pickDirectory(
      initialDirectory: _defaultSaveDirectory(),
      confirmButtonText: '选择保存目录',
    );
    if (saveDirectory == null || saveDirectory.isEmpty) {
      return;
    }

    final outputPath = _buildStegoOutputPath(saveDirectory, carrier, _scene);
    if (_normalizePath(carrier) == _normalizePath(outputPath)) {
      _showMessage('输出不能覆盖载体');
      return;
    }

    setState(() => _isSubmitting = true);
    try {
      final result = switch (_scene) {
        _StegoScene.embed => await _bridge.embedFile(
            carrierPath: carrier,
            payloadPath: _payloadController.text.trim(),
            outputPath: outputPath,
            password: _passwordController.text,
          ),
        _StegoScene.extract => await _bridge.extractFile(
            carrierPath: carrier,
            outputPath: outputPath,
            password: _passwordController.text,
          ),
      };
      _showMessage('${_scene == _StegoScene.embed ? '隐写' : '回显'}完成');
      debugPrint(result.outputPath);
    } catch (error) {
      _showMessage(error.toString());
    } finally {
      if (mounted) setState(() => _isSubmitting = false);
    }
  }

  bool _validateInputs() {
    final carrier = _carrierController.text.trim();
    if (!File(carrier).existsSync()) {
      _showMessage('载体文件不存在');
      return false;
    }
    if (_scene == _StegoScene.embed) {
      final payload = _payloadController.text.trim();
      if (!File(payload).existsSync()) {
        _showMessage('隐藏文件不存在');
        return false;
      }
    }
    return true;
  }

  Future<void> _pickCarrierPath() async {
    final path = await _fileDialog.openFile(initialDirectory: _defaultSaveDirectory());
    if (path != null && mounted) {
      setState(() => _carrierController.text = path);
    }
  }

  Future<void> _pickPayloadPath() async {
    final path = await _fileDialog.openFile(initialDirectory: _defaultSaveDirectory());
    if (path != null && mounted) {
      setState(() => _payloadController.text = path);
    }
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final tiny = constraints.maxHeight < 310;
        final ultra = constraints.maxHeight < 280;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (!tiny) ...[
              const PageHeader(
                title: '文件隐写',
                subtitle: 'Python 方案',
                tag: 'AES-CBC',
                icon: Icons.layers_rounded,
              ),
              const SizedBox(height: 5),
            ],
            Expanded(
              child: SectionCard(
                title: '处理',
                subtitle: tiny ? null : (_scene == _StegoScene.embed ? '隐写' : '回显'),
                badge: _scene == _StegoScene.embed ? 'STG' : 'EXT',
                child: Form(
                  key: _formKey,
                  child: Column(
                    children: [
                      SceneSwitch<_StegoScene>(
                        value: _scene,
                        items: const [
                          SceneSwitchItem(value: _StegoScene.embed, label: '文件隐写'),
                          SceneSwitchItem(value: _StegoScene.extract, label: '隐写回显'),
                        ],
                        onChanged: (value) => setState(() => _scene = value),
                      ),
                      SizedBox(height: ultra ? 4 : 5),
                      DropInputField(
                        controller: _carrierController,
                        label: '载体文件',
                        onBrowse: _pickCarrierPath,
                        validator: _requiredValidator,
                      ),
                      if (_scene == _StegoScene.embed) ...[
                        SizedBox(height: ultra ? 4 : 5),
                        DropInputField(
                          controller: _payloadController,
                          label: '隐藏文件',
                          onBrowse: _pickPayloadPath,
                          validator: _requiredValidator,
                        ),
                      ],
                      SizedBox(height: ultra ? 4 : 5),
                      _passwordField(),
                      const Spacer(),
                      SizedBox(
                        width: double.infinity,
                        child: FilledButton.icon(
                          onPressed: _isSubmitting ? null : _run,
                          icon: Icon(
                            _scene == _StegoScene.embed ? Icons.visibility_off_rounded : Icons.unarchive_rounded,
                            size: 12,
                          ),
                          label: Text(_isSubmitting ? '处理中...' : (_scene == _StegoScene.embed ? '开始隐写' : '开始回显')),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _passwordField() {
    return TextFormField(
      controller: _passwordController,
      obscureText: true,
      decoration: const InputDecoration(labelText: '隐写密码'),
      validator: (value) {
        if (value == null || value.trim().isEmpty) return '请输入隐写密码';
        return null;
      },
    );
  }

  String? _requiredValidator(String? value) {
    if (value == null || value.trim().isEmpty) return '不能为空';
    return null;
  }

  String _defaultSaveDirectory() => File(Platform.resolvedExecutable).parent.path;

  String _buildStegoOutputPath(String directory, String carrierPath, _StegoScene scene) {
    final carrierName = _fileName(carrierPath);
    final outputName = scene == _StegoScene.embed ? '$carrierName.ste' : _restoredName(carrierName);
    return '$directory${Platform.pathSeparator}$outputName';
  }

  String _restoredName(String carrierName) {
    if (carrierName.toLowerCase().endsWith('.ste')) {
      return carrierName.substring(0, carrierName.length - 4);
    }
    return '$carrierName.out';
  }

  String _fileName(String path) {
    final normalized = path.replaceAll('\\', '/');
    return normalized.split('/').last;
  }

  String _normalizePath(String value) => value.trim().replaceAll('/', '\\').toLowerCase();

  void _showMessage(String text) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(text)));
  }
}
