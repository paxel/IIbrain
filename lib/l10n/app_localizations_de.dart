// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get appTitle => 'IIbrain';

  @override
  String get chooseBrainTitle => 'Wo liegt dein Brain?';

  @override
  String get chooseBrainBody =>
      'Dein Brain ist ein Ordner mit Markdown-Dateien. Wähle einen vorhandenen oder einen leeren Ordner. IIbrain synchronisiert ihn nie; ein Sync-Werkzeug deiner Wahl kann ihn zwischen deinen Geräten abgleichen.';

  @override
  String get chooseFolder => 'Ordner wählen…';

  @override
  String get changeFolder => 'Ordner wechseln…';

  @override
  String folderNotOpened(String reason) {
    return 'Dieser Ordner ließ sich nicht öffnen: $reason';
  }

  @override
  String get folders => 'Ordner';

  @override
  String get inbox => 'Eingang';

  @override
  String get brainRoot => 'Brain';

  @override
  String get noNotesHere => 'Hier liegen keine Notizen.';

  @override
  String get chooseNote => 'Wähle eine Notiz, um sie zu lesen.';

  @override
  String get attachments => 'Anhänge';

  @override
  String get extractedText => 'Erkannter Text';

  @override
  String capturedOn(String when) {
    return 'Erfasst $when';
  }

  @override
  String get newNote => 'Neue Notiz';

  @override
  String get noteText => 'Text';

  @override
  String get save => 'Speichern';

  @override
  String get cancel => 'Abbrechen';

  @override
  String get phoneOnly =>
      'Druck-Erfassung, Kamera und Live-Abfragen gibt es auf dem Telefon.';

  @override
  String get edit => 'Bearbeiten';

  @override
  String get fileNote => 'Ablegen…';

  @override
  String get fileNoteTitle => 'Notiz ablegen';

  @override
  String get noteDetails => 'Titel, Tags und Ordner…';

  @override
  String get noteDetailsTitle => 'Titel, Tags und Ordner';

  @override
  String get delete => 'Löschen';

  @override
  String get deleteNoteQuestion => 'Diese Notiz mit ihren Anhängen löschen?';

  @override
  String get titleLabel => 'Titel';

  @override
  String get tagsLabel => 'Tags';

  @override
  String get tagsHint => 'durch Kommas getrennt';

  @override
  String get folderLabel => 'Ordner';

  @override
  String get newFolder => 'Neuer Ordner…';

  @override
  String get newFolderTitle => 'Neuer Ordner';

  @override
  String get renameFolder => 'Umbenennen…';

  @override
  String get renameFolderTitle => 'Ordner umbenennen';

  @override
  String get removeFolder => 'Entfernen';

  @override
  String get folderName => 'Name';

  @override
  String get create => 'Anlegen';

  @override
  String get rename => 'Umbenennen';

  @override
  String unfiledWaiting(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notizen warten auf Ablage.',
      one: '1 Notiz wartet auf Ablage.',
      zero: 'Nichts wartet auf Ablage.',
    );
    return '$_temp0';
  }

  @override
  String errorNotPossible(String reason) {
    return 'Das geht nicht: $reason';
  }

  @override
  String errorNotFound(String what) {
    return 'Nicht gefunden: $what';
  }

  @override
  String errorFile(String reason) {
    return 'Eine Datei ließ sich nicht lesen oder schreiben: $reason';
  }

  @override
  String errorOther(String reason) {
    return 'Etwas ging schief: $reason';
  }

  @override
  String get search => 'Suche';

  @override
  String searchResults(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notizen gefunden.',
      one: '1 Notiz gefunden.',
      zero: 'Nichts gefunden.',
    );
    return '$_temp0';
  }

  @override
  String get clearSearch => 'Leeren';

  @override
  String get addFiles => 'Dateien hinzufügen…';

  @override
  String get noTextFound => 'Darin wurde kein Text gefunden.';

  @override
  String get readingText => 'Der Text wird gelesen…';

  @override
  String get pageText => 'Text der Seite';

  @override
  String get pageTextMissing =>
      'Der Text der Seite fehlt: Sie ließ sich nicht abrufen.';

  @override
  String get pageFetching => 'Die Seite wird abgerufen…';

  @override
  String get linkHint =>
      'Ein Link allein wird zusammen mit dem Text seiner Seite aufbewahrt.';

  @override
  String get models => 'Modelle…';

  @override
  String get modelsTitle => 'Modelle';

  @override
  String get noModels =>
      'Kein Modell ist verbunden. IIbrain bringt keines mit; ohne eines erfasst du, legst von Hand ab und suchst.';

  @override
  String get localModel => 'Lokales Modell';

  @override
  String get cloudModel => 'Cloud-Modell';

  @override
  String get remove => 'Entfernen';

  @override
  String get connectModel => 'Modell verbinden';

  @override
  String get modelName => 'Name';

  @override
  String get modelAddress => 'Adresse';

  @override
  String get modelAddressHint => 'etwa http://localhost:11434/v1';

  @override
  String get modelKey => 'Schlüssel';

  @override
  String get modelId => 'Modell am Endpunkt';

  @override
  String get modelIsLocal =>
      'Es läuft unter meiner eigenen Kontrolle (lokales Modell)';

  @override
  String get modelStaysHere =>
      'Adresse und Schlüssel bleiben auf diesem Gerät und gelangen nie in den Ordner des Brains.';

  @override
  String get testAndAdd => 'Testen und hinzufügen';

  @override
  String get addUntested => 'Ohne Test hinzufügen';

  @override
  String get close => 'Schließen';

  @override
  String errorModel(String reason) {
    return 'Das Modell hat nicht geantwortet: $reason';
  }

  @override
  String get privateMark => 'Privat';

  @override
  String get privateExplain =>
      'Was privat ist, erreicht nur ein lokales Modell, nie ein Cloud-Modell.';

  @override
  String get privateByFolder => 'Privat, weil der Ordner es ist.';

  @override
  String get privateUnfiled => 'Privat, solange sie nicht abgelegt ist.';

  @override
  String get folderMarkPrivate => 'Als privat markieren';

  @override
  String get folderLiftPrivate => 'Privat-Markierung aufheben';

  @override
  String get unfiledPrivateSetting =>
      'Nicht abgelegte Notizen gelten als privat';

  @override
  String privateSuffix(String name) {
    return '$name · privat';
  }

  @override
  String get propose => 'Vorschlagen';

  @override
  String get modelWorking => 'Das Modell arbeitet…';

  @override
  String proposalBy(String model) {
    return 'Vorschlag von $model. Korrigiere ihn oder speichere ihn so.';
  }

  @override
  String get errorNoModel =>
      'Kein Modell ist verbunden. Verbinde eines unter Modelle…, oder mach es von Hand.';

  @override
  String get errorPrivate =>
      'Das ist privat und dein Modell ist ein Cloud-Modell, also wurde nichts gesendet. Wähle ein lokales Modell oder hebe die Privat-Markierung auf.';

  @override
  String newFolderSuffix(String name) {
    return '$name (neuer Ordner)';
  }

  @override
  String get ask => 'Fragen';

  @override
  String get askTitle => 'Frag dein Brain';

  @override
  String get questionLabel => 'Frage';

  @override
  String get backToNotes => 'Zurück zu den Notizen';

  @override
  String get nothingFits =>
      'Nichts in deinem Brain passt zu dieser Frage. Es wurde kein Modell gefragt.';

  @override
  String answerBy(String model) {
    return 'Antwort von $model, allein aus den Quellen unten:';
  }

  @override
  String get sources => 'Quellen';

  @override
  String privateLeftOut(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count private Notizen wurden weggelassen, weil dein Modell ein Cloud-Modell ist. Die Antwort kann unvollständig sein.',
      one: '1 private Notiz wurde weggelassen, weil dein Modell ein Cloud-Modell ist. Die Antwort kann unvollständig sein.',
    );
    return '$_temp0';
  }

  @override
  String get keepAnswer => 'Als Notiz behalten';

  @override
  String writtenByModel(String model) {
    return 'Vom Modell $model geschrieben, aus diesen Quellen:';
  }

  @override
  String answerSuffix(String name) {
    return '$name · Antwort';
  }

  @override
  String sourceGone(String name) {
    return '$name (nicht mehr vorhanden)';
  }

  @override
  String conflictSuffix(String name) {
    return '$name · Konflikt';
  }

  @override
  String conflictBanner(int count) {
    return '$count Fassungen dieser Notiz liegen nebeneinander: Zwei Geräte haben sie geändert, und dein Sync-Werkzeug hat beide behalten.';
  }

  @override
  String get resolveConflict => 'Auflösen…';

  @override
  String get conflictTitle => 'Konflikt';

  @override
  String versionOf(int number) {
    return 'Fassung $number';
  }

  @override
  String keepVersion(int number) {
    return 'Fassung $number behalten';
  }

  @override
  String get mergedText => 'Zusammengeführter Text';

  @override
  String get keepMerged => 'Zusammengeführten Text behalten';

  @override
  String get allowFilesTitle => 'IIbrain braucht Zugriff auf deine Dateien';

  @override
  String get allowFilesBody =>
      'Dein Brain ist ein Ordner im Speicher des Telefons, in den auch dein Sync-Werkzeug schreibt. Um ihn zu lesen und zu schreiben, braucht IIbrain die Erlaubnis, alle Dateien zu verwalten. Es nutzt sie nur für diesen Ordner.';

  @override
  String get allowFilesButton => 'Erlaubnis öffnen';

  @override
  String get useThisFolder => 'Diesen Ordner verwenden';

  @override
  String get folderUp => 'Nach oben';

  @override
  String get noFoldersHere => 'Hier sind keine Ordner.';

  @override
  String get tabBrain => 'Brain';

  @override
  String get tabSearch => 'Suche';

  @override
  String get tabSettings => 'Einstellungen';

  @override
  String get searchHint => 'Tippe Wörter, um dein Brain zu durchsuchen.';

  @override
  String get brainFolder => 'Ordner des Brains';

  @override
  String notesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notizen',
      one: '1 Notiz',
      zero: 'Keine Notizen',
    );
    return '$_temp0';
  }

  @override
  String version(String version) {
    return 'Version $version';
  }

  @override
  String errorIndex(String reason) {
    return 'Der Index der App ließ sich nicht lesen oder schreiben: $reason';
  }

  @override
  String get takePhoto => 'Foto aufnehmen';

  @override
  String removeFile(String name) {
    return '$name entfernen';
  }

  @override
  String keptInInbox(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Erfassungen liegen in deinem Eingang.',
      one: '1 Erfassung liegt in deinem Eingang.',
    );
    return '$_temp0';
  }

  @override
  String get share => 'Teilen';

  @override
  String get printCapture => 'Druck-Erfassung';

  @override
  String get printCaptureBody =>
      'Schalte IIbrain einmal als Druckdienst ein. Wähle dann in jeder App IIbrain als Drucker: Die Seite liegt als PDF in deinem Eingang, sobald du IIbrain das nächste Mal öffnest.';

  @override
  String get openPrintSettings => 'Druckeinstellungen öffnen';

  @override
  String get lookupCalendar => 'Kalender';

  @override
  String get lookupContacts => 'Kontakte';

  @override
  String get lookupExplain =>
      'Jetzt gelesen, nur für diese Frage. Nichts davon bleibt in deinem Brain.';

  @override
  String get lookupRefused =>
      'IIbrain darf das nicht lesen. Erlaube es in den Systemeinstellungen der App.';

  @override
  String get calendarToCloud => 'Kalender darf an Cloud-Modelle gehen';

  @override
  String get contactsToCloud => 'Kontakte dürfen an Cloud-Modelle gehen';

  @override
  String get lookupLocalOnly => 'Aus: Es geht nur an lokale Modelle.';
}
