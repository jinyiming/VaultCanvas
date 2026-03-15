import 'package:flutter/material.dart';

ThemeData buildAppTheme() {
  const seed = Color(0xFF0E7490);
  final scheme = ColorScheme.fromSeed(
    seedColor: seed,
    brightness: Brightness.light,
  );

  OutlineInputBorder border([Color? color]) {
    return OutlineInputBorder(
      borderRadius: BorderRadius.circular(7),
      borderSide: BorderSide(color: color ?? const Color(0xFFD4E3E7)),
    );
  }

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: const Color(0xFFF3F7F8),
    fontFamily: 'Microsoft YaHei UI',
    visualDensity: const VisualDensity(horizontal: -3, vertical: -3),
    textTheme: const TextTheme(
      headlineMedium: TextStyle(fontSize: 16.5, height: 1.0, fontWeight: FontWeight.w700, letterSpacing: 0.05),
      titleLarge: TextStyle(fontSize: 12, height: 1.0, fontWeight: FontWeight.w700, letterSpacing: 0.05),
      titleMedium: TextStyle(fontSize: 11, height: 1.0, fontWeight: FontWeight.w700),
      bodyMedium: TextStyle(fontSize: 9, height: 1.15, fontWeight: FontWeight.w500),
      bodySmall: TextStyle(fontSize: 8, height: 1.1, fontWeight: FontWeight.w500),
      labelLarge: TextStyle(fontSize: 9.5, fontWeight: FontWeight.w700, letterSpacing: 0.1),
    ),
    cardTheme: CardThemeData(
      elevation: 0,
      margin: EdgeInsets.zero,
      color: Colors.white.withValues(alpha: 0.98),
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(11),
        side: const BorderSide(color: Color(0xFFD9E5E8)),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: const Color(0xFFF8FBFB),
      isDense: true,
      contentPadding: const EdgeInsets.symmetric(horizontal: 7, vertical: 6),
      border: border(),
      enabledBorder: border(),
      focusedBorder: border(const Color(0xFF0E7490)),
      errorBorder: border(const Color(0xFFB42318)),
      focusedErrorBorder: border(const Color(0xFFB42318)),
      labelStyle: const TextStyle(fontSize: 9, fontWeight: FontWeight.w600),
      hintStyle: const TextStyle(fontSize: 8.5),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        elevation: 0,
        backgroundColor: const Color(0xFF145D6F),
        foregroundColor: Colors.white,
        minimumSize: const Size(0, 28),
        padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 5),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(7)),
        textStyle: const TextStyle(fontSize: 9.5, fontWeight: FontWeight.w700, letterSpacing: 0.1),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: const Color(0xFF145D6F),
        side: const BorderSide(color: Color(0xFFC9DCE1)),
        minimumSize: const Size(0, 28),
        padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 5),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(7)),
        textStyle: const TextStyle(fontSize: 9.5, fontWeight: FontWeight.w700, letterSpacing: 0.1),
      ),
    ),
    chipTheme: const ChipThemeData(
      backgroundColor: Color(0xFFF2F7F8),
      selectedColor: Color(0xFFD8EEF3),
      side: BorderSide(color: Color(0xFFD4E3E7)),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(7)),
      ),
      labelStyle: TextStyle(fontSize: 8.5, color: Color(0xFF163A43), fontWeight: FontWeight.w600),
      padding: EdgeInsets.symmetric(horizontal: 4, vertical: 1.5),
    ),
    sliderTheme: const SliderThemeData(
      activeTrackColor: Color(0xFF145D6F),
      inactiveTrackColor: Color(0xFFDCE8EB),
      thumbColor: Color(0xFF145D6F),
      overlayColor: Color(0x22145D6F),
      trackHeight: 2,
      thumbShape: RoundSliderThumbShape(enabledThumbRadius: 5.5),
      overlayShape: RoundSliderOverlayShape(overlayRadius: 9),
    ),
    snackBarTheme: SnackBarThemeData(
      behavior: SnackBarBehavior.floating,
      backgroundColor: const Color(0xFF173C45),
      contentTextStyle: const TextStyle(color: Colors.white, fontSize: 8.5),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(7)),
    ),
  );
}
