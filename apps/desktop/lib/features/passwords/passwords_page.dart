import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../services/security_bridge.dart';
import '../../shared/page_header.dart';
import '../../shared/section_card.dart';

class PasswordsPage extends StatefulWidget {
  const PasswordsPage({super.key});

  @override
  State<PasswordsPage> createState() => _PasswordsPageState();
}

class _PasswordsPageState extends State<PasswordsPage> {
  final _bridge = const SecurityBridge();
  final _resultController = TextEditingController(text: '点击生成');

  bool _isGeneratingPassword = false;
  double _length = 18;
  bool _lowercase = true;
  bool _uppercase = true;
  bool _digits = true;
  bool _symbols = true;

  @override
  void dispose() {
    _resultController.dispose();
    super.dispose();
  }

  Future<void> _generatePassword() async {
    if (!_hasEnabledCharset()) {
      _showMessage('请至少启用一种字符');
      return;
    }
    setState(() => _isGeneratingPassword = true);
    try {
      final value = await _bridge.generatePassword(
        PasswordPolicyDto(
          length: _length.round(),
          useLowercase: _lowercase,
          useUppercase: _uppercase,
          useDigits: _digits,
          useSymbols: _symbols,
        ),
      );
      _resultController.text = value;
      _showMessage('已生成');
    } catch (error) {
      _showMessage(error.toString());
    } finally {
      if (mounted) setState(() => _isGeneratingPassword = false);
    }
  }

  Future<void> _copyText() async {
    if (_resultController.text == '点击生成') return;
    await Clipboard.setData(ClipboardData(text: _resultController.text));
    if (!mounted) return;
    _showMessage('已复制');
  }

  void _showMessage(String text) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(text)));
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
                title: '密码生成',
                subtitle: '随机',
                tag: 'GEN',
                icon: Icons.key_rounded,
              ),
              const SizedBox(height: 5),
            ],
            Expanded(
              child: SectionCard(
                title: '随机密码',
                subtitle: tiny ? null : '长度与字符',
                badge: 'PWD',
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _sliderHeader(_length.round()),
                    Slider(
                      value: _length,
                      min: 12,
                      max: 32,
                      divisions: 20,
                      onChanged: (value) => setState(() => _length = value),
                    ),
                    Wrap(
                      spacing: 4,
                      runSpacing: 4,
                      children: [
                        _toggleChip('小写', _lowercase, (value) => setState(() => _lowercase = value)),
                        _toggleChip('大写', _uppercase, (value) => setState(() => _uppercase = value)),
                        _toggleChip('数字', _digits, (value) => setState(() => _digits = value)),
                        _toggleChip('符号', _symbols, (value) => setState(() => _symbols = value)),
                      ],
                    ),
                    SizedBox(height: ultra ? 4 : 5),
                    TextFormField(
                      controller: _resultController,
                      readOnly: true,
                      decoration: const InputDecoration(labelText: '结果'),
                      style: const TextStyle(fontWeight: FontWeight.w700),
                    ),
                    const Spacer(),
                    _ActionBar(
                      busy: _isGeneratingPassword,
                      onPrimary: _generatePassword,
                      onCopy: _resultController.text == '点击生成' ? null : _copyText,
                    ),
                  ],
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _sliderHeader(int value) {
    return Row(
      children: [
        const Text('长度', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 9.5)),
        const Spacer(),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(
            color: const Color(0xFFE7F3F6),
            borderRadius: BorderRadius.circular(999),
          ),
          child: Text(
            '$value',
            style: const TextStyle(fontSize: 8.5, fontWeight: FontWeight.w700, color: Color(0xFF145D6F)),
          ),
        ),
      ],
    );
  }

  Widget _toggleChip(String label, bool selected, ValueChanged<bool> onSelected) {
    return FilterChip(
      label: Text(label),
      selected: selected,
      onSelected: onSelected,
      visualDensity: const VisualDensity(horizontal: -4, vertical: -4),
    );
  }

  bool _hasEnabledCharset() => _lowercase || _uppercase || _digits || _symbols;
}

class _ActionBar extends StatelessWidget {
  const _ActionBar({
    required this.busy,
    required this.onPrimary,
    required this.onCopy,
  });

  final bool busy;
  final VoidCallback onPrimary;
  final VoidCallback? onCopy;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: FilledButton.icon(
            onPressed: busy ? null : onPrimary,
            icon: const Icon(Icons.bolt_rounded, size: 12),
            label: Text(busy ? '生成中...' : '生成'),
          ),
        ),
        const SizedBox(width: 5),
        OutlinedButton.icon(
          onPressed: onCopy,
          icon: const Icon(Icons.copy_rounded, size: 12),
          label: const Text('复制'),
        ),
      ],
    );
  }
}
