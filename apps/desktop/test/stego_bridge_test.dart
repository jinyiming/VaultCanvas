import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:vaultcanvas_desktop/services/security_bridge.dart';

void main() {
  test('security bridge embeds and extracts a file', () async {
    const bridge = SecurityBridge();
    final tempDir = await Directory.systemTemp.createTemp('vaultcanvas_stego_');

    try {
      final carrier = File('${tempDir.path}\\carrier.bin');
      final payload = File('${tempDir.path}\\payload.txt');
      final embedded = File('${tempDir.path}\\carrier_embedded.bin');
      final extracted = File('${tempDir.path}\\payload.restored.txt');

      await carrier.writeAsBytes(List<int>.generate(256, (index) => index % 256));
      await payload.writeAsString('VaultCanvas bridge stego test');

      final embedResult = await bridge.embedFile(
        carrierPath: carrier.path,
        payloadPath: payload.path,
        outputPath: embedded.path,
        password: 'StegoPassword!42',
      );

      expect(embedResult.outputPath, embedded.path);
      expect(await embedded.exists(), isTrue);

      final extractResult = await bridge.extractFile(
        carrierPath: embedded.path,
        outputPath: extracted.path,
        password: 'StegoPassword!42',
      );

      expect(extractResult.outputPath, extracted.path);
      expect(await extracted.exists(), isTrue);
      expect(await extracted.readAsString(), 'VaultCanvas bridge stego test');
    } finally {
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    }
  });
}
