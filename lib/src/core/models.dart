/// What the core hands the app, read from its JSON.
library;

List<String> _strings(Object? value) => [
  if (value is List<Object?>)
    for (final item in value)
      if (item is String) item,
];

/// A file kept with a Note.
class Attachment {
  const Attachment({required this.name, this.extracted});

  factory Attachment.fromJson(Map<String, Object?> json) => Attachment(
    name: json['name'] as String? ?? '',
    extracted: json['extracted'] as String?,
  );

  final String name;

  /// Its Extracted Text: null while nothing was read, empty when the
  /// app read it and found no text.
  final String? extracted;
}

/// One kept thing.
class Note {
  const Note({
    required this.id,
    required this.folder,
    required this.title,
    this.tags = const [],
    this.captured,
    this.via,
    this.link,
    this.private = false,
    this.isPrivate = false,
    this.unfiled = false,
    this.model,
    this.sources = const [],
    this.text = '',
    this.attachments = const [],
    this.pageText,
    this.conflicts = const [],
  });

  factory Note.fromJson(Map<String, Object?> json) => Note(
    id: json['id'] as String? ?? '',
    folder: json['folder'] as String? ?? '',
    title: json['title'] as String? ?? '',
    tags: _strings(json['tags']),
    captured: json['captured'] as String?,
    via: json['via'] as String?,
    link: json['link'] as String?,
    private: json['private'] as bool? ?? false,
    isPrivate: json['is_private'] as bool? ?? false,
    unfiled: json['unfiled'] as bool? ?? false,
    model: json['model'] as String?,
    sources: _strings(json['sources']),
    text: json['text'] as String? ?? '',
    attachments: [
      if (json['attachments'] case final List<Object?> items)
        for (final item in items)
          if (item is Map<String, Object?>) Attachment.fromJson(item),
    ],
    pageText: json['page_text'] as String?,
    conflicts: _strings(json['conflicts']),
  );

  final String id;
  final String folder;
  final String title;
  final List<String> tags;
  final String? captured;
  final String? via;
  final String? link;

  /// Its own Private mark.
  final bool private;

  /// Whether it is Private now: by its own mark, its Folder's, or as
  /// an Unfiled Note while those count as Private.
  final bool isPrivate;
  final bool unfiled;

  /// The Model that wrote it, for a Saved Answer.
  final String? model;
  final List<String> sources;
  final String text;
  final List<Attachment> attachments;
  final String? pageText;

  /// The other versions a Sync Tool left of it; not empty means the
  /// Note is in Conflict.
  final List<String> conflicts;

  /// What it is listed as: its title, or for an Unfiled Note without
  /// one the start of its text.
  String get listTitle {
    final unnamed = unfiled && id.endsWith('/$title.md');
    if (!unnamed) return title;
    for (final line in text.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isNotEmpty) {
        return trimmed.length > 60 ? trimmed.substring(0, 60) : trimmed;
      }
    }
    return title;
  }
}

/// A Folder of the Brain.
class Folder {
  const Folder({required this.name, this.private = false, this.notes = 0});

  factory Folder.fromJson(Map<String, Object?> json) => Folder(
    name: json['name'] as String? ?? '',
    private: json['private'] as bool? ?? false,
    notes: json['notes'] as int? ?? 0,
  );

  final String name;
  final bool private;
  final int notes;
}

/// A Note found by a search.
class Hit {
  const Hit({
    required this.id,
    required this.title,
    required this.folder,
    required this.excerpt,
  });

  factory Hit.fromJson(Map<String, Object?> json) => Hit(
    id: json['id'] as String? ?? '',
    title: json['title'] as String? ?? '',
    folder: json['folder'] as String? ?? '',
    excerpt: json['excerpt'] as String? ?? '',
  );

  final String id;
  final String title;
  final String folder;
  final String excerpt;
}

/// A connected Model.
class Model {
  const Model({
    required this.name,
    required this.address,
    this.key,
    this.model = '',
    this.local = false,
  });

  factory Model.fromJson(Map<String, Object?> json) => Model(
    name: json['name'] as String? ?? '',
    address: json['address'] as String? ?? '',
    key: json['key'] as String?,
    model: json['model'] as String? ?? '',
    local: json['local'] as bool? ?? false,
  );

  final String name;
  final String address;
  final String? key;
  final String model;

  /// Whether the person declared it a Local Model.
  final bool local;

  Map<String, Object?> toJson() => {
    'name': name,
    'address': address,
    'key': key,
    'model': model,
    'local': local,
  };
}

/// What the app knows about the Device and its Brain at a glance.
class BrainState {
  const BrainState({
    this.brain,
    this.models = const [],
    this.chosen,
    this.unfiledPrivate = true,
    this.calendarToCloud = false,
    this.contactsToCloud = false,
    this.version = '',
  });

  factory BrainState.fromJson(Map<String, Object?> json) => BrainState(
    brain: json['brain'] as String?,
    models: [
      if (json['models'] case final List<Object?> items)
        for (final item in items)
          if (item is Map<String, Object?>) Model.fromJson(item),
    ],
    chosen: json['chosen'] as String?,
    unfiledPrivate: json['unfiled_private'] as bool? ?? true,
    calendarToCloud: json['calendar_to_cloud'] as bool? ?? false,
    contactsToCloud: json['contacts_to_cloud'] as bool? ?? false,
    version: json['version'] as String? ?? '',
  );

  /// The Brain's folder, once one is chosen and within reach.
  final String? brain;
  final List<Model> models;

  /// The name of the Model that proposes and answers.
  final String? chosen;
  final bool unfiledPrivate;
  final bool calendarToCloud;
  final bool contactsToCloud;
  final String version;
}

/// What a Model suggests for an Unfiled Note.
class Proposal {
  const Proposal({
    required this.title,
    this.tags = const [],
    this.folder = '',
    this.newFolder = false,
    this.model = '',
  });

  factory Proposal.fromJson(Map<String, Object?> json) => Proposal(
    title: json['title'] as String? ?? '',
    tags: _strings(json['tags']),
    folder: json['folder'] as String? ?? '',
    newFolder: json['new_folder'] as bool? ?? false,
    model: json['model'] as String? ?? '',
  );

  final String title;
  final List<String> tags;
  final String folder;

  /// Whether [folder] does not exist yet.
  final bool newFolder;

  /// The name of the Model that made it.
  final String model;
}

/// A Note handed to the Model for a Question.
class Source {
  const Source({required this.id, required this.title, this.folder = ''});

  factory Source.fromJson(Map<String, Object?> json) => Source(
    id: json['id'] as String? ?? '',
    title: json['title'] as String? ?? '',
    folder: json['folder'] as String? ?? '',
  );

  final String id;
  final String title;
  final String folder;
}

/// What a Model wrote in reply to a Question, from the Sources alone.
class Answer {
  const Answer({
    required this.text,
    this.sources = const [],
    this.model = '',
    this.leftOut = 0,
    this.json = const {},
  });

  factory Answer.fromJson(Map<String, Object?> json) => Answer(
    text: json['text'] as String? ?? '',
    sources: [
      if (json['sources'] case final List<Object?> items)
        for (final item in items)
          if (item is Map<String, Object?>) Source.fromJson(item),
    ],
    model: json['model'] as String? ?? '',
    leftOut: json['left_out'] as int? ?? 0,
    json: json,
  );

  final String text;
  final List<Source> sources;
  final String model;

  /// How many fitting Private Notes were not handed to a Cloud Model.
  final int leftOut;

  /// The Answer as the core gave it, to hand back when it is kept.
  final Map<String, Object?> json;
}

/// What a Question came to: an [answer], or nothing the Model may see,
/// with [leftOut] fitting Private Notes kept from a Cloud Model.
class Asked {
  Asked.answer(Answer this.answer) : leftOut = answer.leftOut;
  const Asked.nothing(this.leftOut) : answer = null;

  final Answer? answer;
  final int leftOut;
}

/// Something a Live Lookup read on the phone for one Question.
class Lookup {
  const Lookup({required this.kind, required this.text});

  /// `calendar` or `contacts`.
  final String kind;
  final String text;

  Map<String, Object?> toJson() => {'kind': kind, 'text': text};
}
