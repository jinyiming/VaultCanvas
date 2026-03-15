import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:vaultcanvas_desktop/services/security_bridge.dart';

void main() {
  test('security bridge encrypts and decrypts a file', () async {
    const bridge = SecurityBridge();
    final tempDir = await Directory.systemTemp.createTemp('vaultcanvas_crypto_');

    try {
      final input = File('${tempDir.path}\\plain.txt');
      final encrypted = File('${tempDir.path}\\plain.vlt');
      final decrypted = File('${tempDir.path}\\plain.dec.txt');
      await input.writeAsString('VaultCanvas bridge crypto test');

      final encryptResult = await bridge.encryptFile(
        inputPath: input.path,
        outputPath: encrypted.path,
        password: 'CorrectHorseBatteryStaple!23',
        idPassword: 'BridgeId!42',
      );

      expect(encryptResult.outputPath, encrypted.path);
      expect(await encrypted.exists(), isTrue);

      final decryptResult = await bridge.decryptFile(
        inputPath: encrypted.path,
        outputPath: decrypted.path,
        password: 'CorrectHorseBatteryStaple!23',
        idPassword: 'BridgeId!42',
      );

      expect(decryptResult.outputPath, decrypted.path);
      expect(await decrypted.exists(), isTrue);
      expect(await decrypted.readAsString(), 'VaultCanvas bridge crypto test');
    } finally {
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    }
  });
}
