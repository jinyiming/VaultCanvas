import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';

class DropInputField extends StatefulWidget {
  const DropInputField({
    required this.controller,
    required this.label,
    required this.onBrowse,
    this.onFileDropped,
    this.validator,
    this.hintText,
    super.key,
  });

  final TextEditingController controller;
  final String label;
  final String? hintText;
  final Future<void> Function() onBrowse;
  final ValueChanged<String>? onFileDropped;
  final String? Function(String?)? validator;

  @override
  State<DropInputField> createState() => _DropInputFieldState();
}

class _DropInputFieldState extends State<DropInputField> {
  bool _dragging = false;

  @override
  Widget build(BuildContext context) {
    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (detail) {
        setState(() => _dragging = false);
        if (detail.files.isEmpty) {
          return;
        }
        final path = detail.files.first.path;
        widget.controller.text = path;
        widget.onFileDropped?.call(path);
      },
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 160),
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: _dragging ? const Color(0xFFDCEEF3) : const Color(0xFFF7FAFB),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(
            color: _dragging ? const Color(0xFF0E7490) : const Color(0xFFD5E1E5),
            width: _dragging ? 2 : 1,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: TextFormField(
                    controller: widget.controller,
                    maxLines: 1,
                    decoration: InputDecoration(
                      isDense: true,
                      labelText: widget.label,
                      hintText: widget.hintText,
                      border: const OutlineInputBorder(),
                    ),
                    validator: widget.validator,
                  ),
                ),
                const SizedBox(width: 5),
                OutlinedButton(
                  onPressed: widget.onBrowse,
                  child: const Text('选'),
                ),
              ],
            ),
            const SizedBox(height: 3),
            Text(
              _dragging ? '松开填入路径' : '支持拖拽',
              style: TextStyle(
                color: _dragging ? const Color(0xFF0E7490) : const Color(0xFF60747B),
                fontSize: 9,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
