import 'package:file_selector/file_selector.dart' as fs;

class FileDialogService {
  const FileDialogService();

  Future<String?> openFile({
    String? initialDirectory,
    List<fs.XTypeGroup> acceptedTypeGroups = const [],
  }) async {
    final file = await fs.openFile(
      initialDirectory: initialDirectory,
      acceptedTypeGroups: acceptedTypeGroups,
    );
    return file?.path;
  }

  Future<String?> saveFile({
    String? suggestedName,
    String? initialDirectory,
    List<fs.XTypeGroup> acceptedTypeGroups = const [],
  }) async {
    final location = await fs.getSaveLocation(
      suggestedName: suggestedName,
      initialDirectory: initialDirectory,
      acceptedTypeGroups: acceptedTypeGroups,
    );
    return location?.path;
  }

  Future<String?> pickDirectory({
    String? initialDirectory,
    String? confirmButtonText,
  }) {
    return fs.getDirectoryPath(
      initialDirectory: initialDirectory,
      confirmButtonText: confirmButtonText,
    );
  }
}
