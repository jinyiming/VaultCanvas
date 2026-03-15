import 'package:flutter/material.dart';

class SceneSwitchItem<T> {
  const SceneSwitchItem({
    required this.value,
    required this.label,
  });

  final T value;
  final String label;
}

class SceneSwitch<T> extends StatelessWidget {
  const SceneSwitch({
    required this.value,
    required this.items,
    required this.onChanged,
    super.key,
  });

  final T value;
  final List<SceneSwitchItem<T>> items;
  final ValueChanged<T> onChanged;

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        padding: const EdgeInsets.all(2.5),
        decoration: BoxDecoration(
          color: const Color(0xFFF3F7F8),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: const Color(0xFFD7E4E7)),
        ),
        child: Wrap(
          spacing: 2.5,
          runSpacing: 2.5,
          children: [
            for (final item in items)
              ChoiceChip(
                label: Text(item.label),
                selected: item.value == value,
                onSelected: (_) => onChanged(item.value),
                visualDensity: const VisualDensity(horizontal: -3, vertical: -3),
              ),
          ],
        ),
      ),
    );
  }
}
