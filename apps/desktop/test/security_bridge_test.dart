import 'package:flutter_test/flutter_test.dart';
import 'package:vaultcanvas_desktop/services/security_bridge.dart';

void main() {
  test('security bridge generates a password', () async {
    const bridge = SecurityBridge();
    final password = await bridge.generatePassword(
      const PasswordPolicyDto(
        length: 16,
        useLowercase: true,
        useUppercase: true,
        useDigits: true,
        useSymbols: true,
      ),
    );

    expect(password.length, 16);
  });
}
