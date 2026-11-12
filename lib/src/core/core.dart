import 'calls.dart';
import 'models.dart';

export 'calls.dart' show CoreCalls, CoreError;
export 'models.dart';

/// The core as the screens use it: each of its calls by name, with what
/// it gives read into the app's own types.
class Core {
  const Core(this._calls);

  final CoreCalls _calls;

  Future<Object?> _call(
    String name, [
    Map<String, Object?> arguments = const {},
  ]) => callCore(_calls, name, arguments);

  static Map<String, Object?> _map(Object? value) =>
      value is Map<String, Object?> ? value : const {};

  static List<Map<String, Object?>> _maps(Object? value) => [
    if (value is List<Object?>)
      for (final item in value)
        if (item is Map<String, Object?>) item,
  ];

  // ----------------------------------------------------------- Device

  /// Opens the Device whose own files lie in [deviceDir], and its Brain
  /// when it has chosen one that is within reach.
  Future<BrainState> open(String deviceDir) async =>
      BrainState.fromJson(_map(await _call('open', {'device_dir': deviceDir})));

  Future<BrainState> state() async =>
      BrainState.fromJson(_map(await _call('state')));

  /// Makes [folder] the Brain.
  Future<BrainState> chooseBrain(String folder) async => BrainState.fromJson(
    _map(await _call('choose_brain', {'folder': folder})),
  );

  /// Takes in what other tools changed in the Brain's folder.
  Future<void> refresh() async {
    await _call('refresh');
  }

  // ---------------------------------------------------------- reading

  Future<List<Folder>> folders() async => [
    for (final item in _maps(await _call('folders'))) Folder.fromJson(item),
  ];

  /// The Notes lying directly in [folder], newest Capture first.
  Future<List<Note>> notes(String folder) async => [
    for (final item in _maps(await _call('notes', {'folder': folder})))
      Note.fromJson(item),
  ];

  Future<Note> note(String id) async =>
      Note.fromJson(_map(await _call('note', {'id': id})));

  Future<List<Hit>> search(String query) async => [
    for (final item in _maps(await _call('search', {'query': query})))
      Hit.fromJson(item),
  ];

  // ---------------------------------------------------------- writing

  /// Keeps a Capture as an Unfiled Note and says its id.
  Future<String> capture({
    String text = '',
    required String via,
    String? link,
    bool private = false,
    List<String> files = const [],
  }) async =>
      await _call('capture', {
        'capture': {
          'text': text,
          'via': via,
          'link': link,
          'private': private,
          'files': files,
        },
      }) as String? ??
      '';

  /// Gives a Note its title, tags and Folder and says its id from then
  /// on.
  Future<String> file(
    String id,
    String title,
    List<String> tags,
    String folder,
  ) async =>
      await _call('file', {
        'id': id,
        'title': title,
        'tags': tags,
        'folder': folder,
      }) as String? ??
      '';

  Future<void> setText(String id, String text) async {
    await _call('set_text', {'id': id, 'text': text});
  }

  Future<void> delete(String id) async {
    await _call('delete', {'id': id});
  }

  Future<void> createFolder(String name) async {
    await _call('create_folder', {'name': name});
  }

  Future<void> renameFolder(String name, String newName) async {
    await _call('rename_folder', {'name': name, 'new_name': newName});
  }

  Future<void> removeFolder(String name) async {
    await _call('remove_folder', {'name': name});
  }

  /// Reads the Extracted Text of one Attachment, or fetches one page,
  /// that still waits for it; says whether there was one.
  Future<bool> extractOne() async =>
      await _call('extract_one') as bool? ?? false;

  // ---------------------------------------------------------- privacy

  Future<void> setPrivate(String id, {required bool private}) async {
    await _call('set_private', {'id': id, 'private': private});
  }

  Future<void> setFolderPrivate(String folder, {required bool private}) async {
    await _call('set_folder_private', {'folder': folder, 'private': private});
  }

  Future<BrainState> setUnfiledPrivate({required bool private}) async =>
      BrainState.fromJson(
        _map(await _call('set_unfiled_private', {'private': private})),
      );

  /// Opens or closes the switch that lets what a Live Lookup read, in
  /// the `calendar` or the `contacts`, go to a Cloud Model.
  Future<BrainState> setLookupToCloud(
    String kind, {
    required bool allowed,
  }) async => BrainState.fromJson(
    _map(
      await _call('set_lookup_to_cloud', {'kind': kind, 'allowed': allowed}),
    ),
  );

  // ----------------------------------------------------------- models

  Future<BrainState> saveModel(Model model) async => BrainState.fromJson(
    _map(await _call('save_model', {'model': model.toJson()})),
  );

  /// Tries a Model's connection; throws when it does not answer.
  Future<void> testModel(Model model) async {
    await _call('test_model', {'model': model.toJson()});
  }

  Future<BrainState> removeModel(String name) async =>
      BrainState.fromJson(_map(await _call('remove_model', {'name': name})));

  Future<BrainState> chooseModel(String name) async =>
      BrainState.fromJson(_map(await _call('choose_model', {'name': name})));

  /// Asks the chosen Model for a Proposal for the Unfiled Note [id].
  Future<Proposal> propose(String id) async =>
      Proposal.fromJson(_map(await _call('propose', {'id': id})));

  /// Asks [question]; [lookups] is what the phone read live for it.
  Future<Asked> ask(String question, {List<Lookup> lookups = const []}) async {
    final asked = _map(
      await _call('ask', {
        'question': question,
        'lookups': [for (final lookup in lookups) lookup.toJson()],
      }),
    );
    final answer = asked['answer'];
    return answer is Map<String, Object?>
        ? Asked.answer(Answer.fromJson(answer))
        : Asked.nothing(asked['nothing'] as int? ?? 0);
  }

  /// Keeps an Answer as a Saved Answer and says its id.
  Future<String> saveAnswer(Answer answer) async =>
      await _call('save_answer', {'answer': answer.json}) as String? ?? '';

  // -------------------------------------------------------- conflicts

  /// The versions of a Note in Conflict, its own file first.
  Future<List<Note>> versions(String id) async => [
    for (final item in _maps(await _call('versions', {'id': id})))
      Note.fromJson(item),
  ];

  Future<void> keepVersion(String id, String version) async {
    await _call('keep_version', {'id': id, 'version': version});
  }

  Future<void> resolve(String id, String text) async {
    await _call('resolve', {'id': id, 'text': text});
  }
}
