import 'dart:io';

import 'package:flutter/material.dart';

import '../../services/file_dialog_service.dart';
import '../../services/security_bridge.dart';
import '../../shared/drop_input_field.dart';
import '../../shared/page_header.dart';
import '../../shared/scene_switch.dart';
import '../../shared/section_card.dart';

class CryptoPage extends StatefulWidget {
  const CryptoPage({super.key});

  @override
  State<CryptoPage> createState() => _CryptoPageState();
}

enum _CryptoScene { encrypt, decrypt }

class _CryptoPageState extends State<CryptoPage> {
  final _bridge = const SecurityBridge();
  final _fileDialog = const FileDialogService();
  final _formKey = GlobalKey<FormState>();
  final _inputController = TextEditingController();
  final _passwordController = TextEditingController();
  final _idPasswordController = TextEditingController();

  _CryptoScene _scene = _CryptoScene.encrypt;
  bool _isSubmitting = false;

  @override
  void dispose() {
    _inputController.dispose();
    _passwordController.dispose();
    _idPasswordController.dispose();
    super.dispose();
  }

  Future<void> _run() async {
    final inputPath = _inputController.text.trim();
    if (!_formKey.currentState!.validate() || !_validateInput(inputPath)) {
      return;
    }

    final saveDirectory = await _fileDialog.pickDirectory(
      initialDirectory: _defaultSaveDirectory(),
      confirmButtonText: '选择保存目录',
    );
    if (saveDirectory == null || saveDirectory.isEmpty) {
      return;
    }

    final outputPath = _buildCryptoOutputPath(saveDirectory, inputPath, _scene);
    if (_normalizePath(inputPath) == _normalizePath(outputPath)) {
      _showMessage('输出不能覆盖原文件');
      return;
    }

    setState(() => _isSubmitting = true);
    try {
      final result = switch (_scene) {
        _CryptoScene.encrypt => await _bridge.encryptFile(
            inputPath: inputPath,
            outputPath: outputPath,
            password: _passwordController.text,
            idPassword: _idPasswordController.text,
          ),
        _CryptoScene.decrypt => await _bridge.decryptFile(
            inputPath: inputPath,
            outputPath: outputPath,
            password: _passwordController.text,
            idPassword: _idPasswordController.text,
          ),
      };
      _showMessage('${_scene == _CryptoScene.encrypt ? '加密' : '解密'}完成');
      debugPrint(result.outputPath);
    } catch (error) {
      _showMessage(error.toString());
    } finally {
      if (mounted) setState(() => _isSubmitting = false);
    }
  }

  bool _validateInput(String inputPath) {
    if (!File(inputPath).existsSync()) {
      _showMessage(_scene == _CryptoScene.encrypt ? '源文件不存在' : '加密文件不存在');
      return false;
    }
    return true;
  }

  Future<void> _pickInputPath() async {
    final path = await _fileDialog.openFile(initialDirectory: _defaultSaveDirectory());
    if (path != null && mounted) {
      setState(() => _inputController.text = path);
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
                title: '文件加解密',
                subtitle: 'Python V5',
                tag: 'AES-GCM',
                icon: Icons.lock_rounded,
              ),
              const SizedBox(height: 5),
            ],
            Expanded(
              child: SectionCard(
                title: '处理',
                subtitle: tiny ? null : (_scene == _CryptoScene.encrypt ? '加密' : '解密'),
                badge: _scene == _CryptoScene.encrypt ? 'ENC' : 'DEC',
                child: Form(
                  key: _formKey,
                  child: Column(
                    children: [
                      SceneSwitch<_CryptoScene>(
                        value: _scene,
                        items: const [
                          SceneSwitchItem(value: _CryptoScene.encrypt, label: '文件加密'),
                          SceneSwitchItem(value: _CryptoScene.decrypt, label: '文件解密'),
                        ],
                        onChanged: (value) => setState(() => _scene = value),
                      ),
                      SizedBox(height: ultra ? 4 : 5),
                      DropInputField(
                        controller: _inputController,
                        label: _scene == _CryptoScene.encrypt ? '源文件' : '加密文件',
                        onBrowse: _pickInputPath,
                        validator: _requiredValidator,
                      ),
                      SizedBox(height: ultra ? 4 : 5),
                      Row(
                        children: [
                          Expanded(child: _passwordField(_passwordController, '主密码')),
                          const SizedBox(width: 5),
                          Expanded(child: _passwordField(_idPasswordController, 'ID 密码')),
                        ],
                      ),
                      const Spacer(),
                      SizedBox(
                        width: double.infinity,
                        child: FilledButton.icon(
                          onPressed: _isSubmitting ? null : _run,
                          icon: Icon(
                            _scene == _CryptoScene.encrypt ? Icons.lock_rounded : Icons.lock_open_rounded,
                            size: 12,
                          ),
                          label: Text(_isSubmitting ? '处理中...' : (_scene == _CryptoScene.encrypt ? '开始加密' : '开始解密')),
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

  Widget _passwordField(TextEditingController controller, String label) {
    return TextFormField(
      controller: controller,
      obscureText: true,
      decoration: InputDecoration(labelText: label),
      validator: (value) {
        if (value == null || value.trim().isEmpty) return '请输入$label';
        return null;
      },
    );
  }

  String? _requiredValidator(String? value) {
    if (value == null || value.trim().isEmpty) return '不能为空';
    return null;
  }

  String _defaultSaveDirectory() => File(Platform.resolvedExecutable).parent.path;

  String _buildCryptoOutputPath(String directory, String inputPath, _CryptoScene scene) {
    final name = _fileName(inputPath);
    return '$directory${Platform.pathSeparator}${scene == _CryptoScene.encrypt ? '$name.enc' : _decryptedName(name)}';
  }

  String _decryptedName(String name) {
    if (name.toLowerCase().endsWith('.enc')) {
      return name.substring(0, name.length - 4);
    }
    return '$name.dec';
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
