import 'dart:async';
import 'dart:io';

import 'generated/api.dart' as bridge_api;
import 'generated/api/types.dart' as bridge_types;
import 'generated/frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

class PasswordPolicyDto {
  const PasswordPolicyDto({
    required this.length,
    required this.useLowercase,
    required this.useUppercase,
    required this.useDigits,
    required this.useSymbols,
  });

  final int length;
  final bool useLowercase;
  final bool useUppercase;
  final bool useDigits;
  final bool useSymbols;
}

class PassphraseRequestDto {
  const PassphraseRequestDto({
    required this.wordCount,
    required this.separator,
    required this.capitalizeWords,
    required this.appendNumber,
  });

  final int wordCount;
  final String separator;
  final bool capitalizeWords;
  final bool appendNumber;
}

enum EncryptionAlgorithmDto { aes256Gcm }

enum StegoModeDto { append }

class OperationResultDto {
  const OperationResultDto({
    required this.outputPath,
    required this.bytesProcessed,
  });

  final String outputPath;
  final int bytesProcessed;
}

class SecurityBridge {
  const SecurityBridge();

  static Completer<void>? _initCompleter;

  Future<String> generatePassword(PasswordPolicyDto policy) async {
    await _ensureInitialized();
    return bridge_api.generatePassword(
      policy: bridge_types.BridgePasswordPolicy(
        length: BigInt.from(policy.length),
        useLowercase: policy.useLowercase,
        useUppercase: policy.useUppercase,
        useDigits: policy.useDigits,
        useSymbols: policy.useSymbols,
      ),
    );
  }

  Future<String> generatePassphrase(PassphraseRequestDto request) async {
    await _ensureInitialized();
    return bridge_api.generatePassphrase(
      request: bridge_types.BridgePassphraseRequest(
        wordCount: BigInt.from(request.wordCount),
        separator: request.separator,
        capitalizeWords: request.capitalizeWords,
        appendNumber: request.appendNumber,
      ),
    );
  }

  Future<OperationResultDto> encryptFile({
    required String inputPath,
    required String outputPath,
    required String password,
    required String idPassword,
    EncryptionAlgorithmDto algorithm = EncryptionAlgorithmDto.aes256Gcm,
  }) async {
    await _ensureInitialized();
    final result = await bridge_api.encryptFile(
      request: bridge_types.BridgeEncryptRequest(
        inputPath: inputPath,
        outputPath: outputPath,
        password: password,
        idPassword: idPassword,
        algorithm: _mapAlgorithm(algorithm),
      ),
    );
    return _mapResult(result);
  }

  Future<OperationResultDto> decryptFile({
    required String inputPath,
    required String outputPath,
    required String password,
    required String idPassword,
  }) async {
    await _ensureInitialized();
    final result = await bridge_api.decryptFile(
      request: bridge_types.BridgeDecryptRequest(
        inputPath: inputPath,
        outputPath: outputPath,
        password: password,
        idPassword: idPassword,
      ),
    );
    return _mapResult(result);
  }

  Future<OperationResultDto> embedFile({
    required String carrierPath,
    required String payloadPath,
    required String outputPath,
    required String password,
    StegoModeDto mode = StegoModeDto.append,
  }) async {
    await _ensureInitialized();
    final result = await bridge_api.embedFile(
      request: bridge_types.BridgeEmbedRequest(
        carrierPath: carrierPath,
        payloadPath: payloadPath,
        outputPath: outputPath,
        password: password,
        mode: _mapStegoMode(mode),
      ),
    );
    return _mapResult(result);
  }

  Future<OperationResultDto> extractFile({
    required String carrierPath,
    required String outputPath,
    required String password,
    StegoModeDto mode = StegoModeDto.append,
  }) async {
    await _ensureInitialized();
    final result = await bridge_api.extractFile(
      request: bridge_types.BridgeExtractRequest(
        carrierPath: carrierPath,
        outputPath: outputPath,
        password: password,
        mode: _mapStegoMode(mode),
      ),
    );
    return _mapResult(result);
  }

  Future<void> _ensureInitialized() {
    final existing = _initCompleter;
    if (existing != null) {
      return existing.future;
    }

    final completer = Completer<void>();
    _initCompleter = completer;
    RustLib.init(
      externalLibrary: ExternalLibrary.open(_resolveLibraryPath()),
    ).then(completer.complete).catchError((Object error, StackTrace stackTrace) {
      _initCompleter = null;
      completer.completeError(error, stackTrace);
    });
    return completer.future;
  }

  bridge_types.BridgeEncryptionAlgorithm _mapAlgorithm(EncryptionAlgorithmDto value) {
    switch (value) {
      case EncryptionAlgorithmDto.aes256Gcm:
        return bridge_types.BridgeEncryptionAlgorithm.aes256Gcm;
    }
  }

  bridge_types.BridgeStegoMode _mapStegoMode(StegoModeDto value) {
    switch (value) {
      case StegoModeDto.append:
        return bridge_types.BridgeStegoMode.append;
    }
  }

  OperationResultDto _mapResult(bridge_types.BridgeOperationResult result) {
    return OperationResultDto(
      outputPath: result.outputPath,
      bytesProcessed: result.bytesProcessed.toInt(),
    );
  }

  String _resolveLibraryPath() {
    const fileName = 'security_core.dll';

    final executableDir = File(Platform.resolvedExecutable).parent;
    final besideExecutable = File('${executableDir.path}\\$fileName');
    if (besideExecutable.existsSync()) {
      return besideExecutable.path;
    }

    final currentDir = Directory.current;
    final repoCandidate = File(
      '${currentDir.parent.parent.path}\\target\\release\\$fileName',
    );
    if (repoCandidate.existsSync()) {
      return repoCandidate.path;
    }

    return fileName;
  }
}
