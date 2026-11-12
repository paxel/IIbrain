// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'IIbrain';

  @override
  String get chooseBrainTitle => 'Where is your Brain?';

  @override
  String get chooseBrainBody =>
      'Your Brain is a folder of Markdown files. Choose an existing folder or an empty one. IIbrain never syncs it; a sync tool of your choice can carry it between your devices.';

  @override
  String get chooseFolder => 'Choose folder…';

  @override
  String get changeFolder => 'Change folder…';

  @override
  String folderNotOpened(String reason) {
    return 'This folder could not be opened: $reason';
  }

  @override
  String get folders => 'Folders';

  @override
  String get inbox => 'Inbox';

  @override
  String get brainRoot => 'Brain';

  @override
  String get noNotesHere => 'No Notes here.';

  @override
  String get chooseNote => 'Choose a Note to read it.';

  @override
  String get attachments => 'Attachments';

  @override
  String get extractedText => 'Extracted Text';

  @override
  String capturedOn(String when) {
    return 'Captured $when';
  }

  @override
  String get newNote => 'New Note';

  @override
  String get noteText => 'Text';

  @override
  String get save => 'Save';

  @override
  String get cancel => 'Cancel';

  @override
  String get phoneOnly =>
      'Print Capture, the camera and Live Lookups are found on the phone.';

  @override
  String get edit => 'Edit';

  @override
  String get fileNote => 'File…';

  @override
  String get fileNoteTitle => 'File this Note';

  @override
  String get noteDetails => 'Title, tags and Folder…';

  @override
  String get noteDetailsTitle => 'Title, tags and Folder';

  @override
  String get delete => 'Delete';

  @override
  String get deleteNoteQuestion => 'Delete this Note with its Attachments?';

  @override
  String get titleLabel => 'Title';

  @override
  String get tagsLabel => 'Tags';

  @override
  String get tagsHint => 'separated by commas';

  @override
  String get folderLabel => 'Folder';

  @override
  String get newFolder => 'New Folder…';

  @override
  String get newFolderTitle => 'New Folder';

  @override
  String get renameFolder => 'Rename…';

  @override
  String get renameFolderTitle => 'Rename Folder';

  @override
  String get removeFolder => 'Remove';

  @override
  String get folderName => 'Name';

  @override
  String get create => 'Create';

  @override
  String get rename => 'Rename';

  @override
  String unfiledWaiting(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notes wait Unfiled.',
      one: '1 Note waits Unfiled.',
      zero: 'Nothing waits Unfiled.',
    );
    return '$_temp0';
  }

  @override
  String errorNotPossible(String reason) {
    return 'That is not possible: $reason';
  }

  @override
  String errorNotFound(String what) {
    return 'Not found: $what';
  }

  @override
  String errorFile(String reason) {
    return 'A file could not be read or written: $reason';
  }

  @override
  String errorOther(String reason) {
    return 'Something went wrong: $reason';
  }

  @override
  String get search => 'Search';

  @override
  String searchResults(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notes found.',
      one: '1 Note found.',
      zero: 'Nothing found.',
    );
    return '$_temp0';
  }

  @override
  String get clearSearch => 'Clear';

  @override
  String get addFiles => 'Add files…';

  @override
  String get noTextFound => 'No text was found in it.';

  @override
  String get readingText => 'Its text is being read…';

  @override
  String get pageText => 'Text of the page';

  @override
  String get pageTextMissing =>
      'The text of the page is missing: it could not be fetched.';

  @override
  String get pageFetching => 'The page is being fetched…';

  @override
  String get linkHint =>
      'A link on its own is kept together with the text of its page.';

  @override
  String get models => 'Models…';

  @override
  String get modelsTitle => 'Models';

  @override
  String get noModels =>
      'No Model is connected. IIbrain brings none of its own; without one you capture, file by hand and search.';

  @override
  String get localModel => 'Local Model';

  @override
  String get cloudModel => 'Cloud Model';

  @override
  String get remove => 'Remove';

  @override
  String get connectModel => 'Connect a Model';

  @override
  String get modelName => 'Name';

  @override
  String get modelAddress => 'Address';

  @override
  String get modelAddressHint => 'such as http://localhost:11434/v1';

  @override
  String get modelKey => 'Key';

  @override
  String get modelId => 'Model at the endpoint';

  @override
  String get modelIsLocal => 'It runs under my own control (Local Model)';

  @override
  String get modelStaysHere =>
      'Address and key stay on this device and never enter the Brain\'s folder.';

  @override
  String get testAndAdd => 'Test and add';

  @override
  String get addUntested => 'Add without testing';

  @override
  String get close => 'Close';

  @override
  String errorModel(String reason) {
    return 'The Model did not answer: $reason';
  }

  @override
  String get privateMark => 'Private';

  @override
  String get privateExplain =>
      'What is Private reaches only a Local Model, never a Cloud Model.';

  @override
  String get privateByFolder => 'Private because its Folder is.';

  @override
  String get privateUnfiled => 'Private as long as it is Unfiled.';

  @override
  String get folderMarkPrivate => 'Mark Private';

  @override
  String get folderLiftPrivate => 'Lift Private mark';

  @override
  String get unfiledPrivateSetting => 'Unfiled Notes count as Private';

  @override
  String privateSuffix(String name) {
    return '$name · Private';
  }

  @override
  String get propose => 'Propose';

  @override
  String get modelWorking => 'The Model is working…';

  @override
  String proposalBy(String model) {
    return 'Proposal by $model. Correct it or save it as it is.';
  }

  @override
  String get errorNoModel =>
      'No Model is connected. Connect one under Models…, or do this by hand.';

  @override
  String get errorPrivate =>
      'This is Private and your Model is a Cloud Model, so nothing was sent. Choose a Local Model, or lift the Private mark.';

  @override
  String newFolderSuffix(String name) {
    return '$name (new Folder)';
  }

  @override
  String get ask => 'Ask';

  @override
  String get askTitle => 'Ask your Brain';

  @override
  String get questionLabel => 'Question';

  @override
  String get backToNotes => 'Back to the Notes';

  @override
  String get nothingFits =>
      'Nothing in your Brain fits this Question. No Model was asked.';

  @override
  String answerBy(String model) {
    return 'Answer by $model, from the Sources below alone:';
  }

  @override
  String get sources => 'Sources';

  @override
  String privateLeftOut(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count Private Notes were left out, because your Model is a Cloud Model. The Answer may be incomplete.',
      one: '1 Private Note was left out, because your Model is a Cloud Model. The Answer may be incomplete.',
    );
    return '$_temp0';
  }

  @override
  String get keepAnswer => 'Keep as a Note';

  @override
  String writtenByModel(String model) {
    return 'Written by the Model $model, from these Sources:';
  }

  @override
  String answerSuffix(String name) {
    return '$name · Answer';
  }

  @override
  String sourceGone(String name) {
    return '$name (no longer there)';
  }

  @override
  String conflictSuffix(String name) {
    return '$name · Conflict';
  }

  @override
  String conflictBanner(int count) {
    return '$count versions of this Note lie side by side: two devices changed it and your sync tool kept both.';
  }

  @override
  String get resolveConflict => 'Resolve…';

  @override
  String get conflictTitle => 'Conflict';

  @override
  String versionOf(int number) {
    return 'Version $number';
  }

  @override
  String keepVersion(int number) {
    return 'Keep version $number';
  }

  @override
  String get mergedText => 'Merged text';

  @override
  String get keepMerged => 'Keep the merged text';

  @override
  String get allowFilesTitle => 'IIbrain needs access to your files';

  @override
  String get allowFilesBody =>
      'Your Brain is a folder in the phone\'s storage that your sync tool also writes to. To read and write it, IIbrain needs the permission to manage all files. It uses it for that folder only.';

  @override
  String get allowFilesButton => 'Open the permission';

  @override
  String get useThisFolder => 'Use this folder';

  @override
  String get folderUp => 'Up';

  @override
  String get noFoldersHere => 'No folders in here.';

  @override
  String get tabBrain => 'Brain';

  @override
  String get tabSearch => 'Search';

  @override
  String get tabSettings => 'Settings';

  @override
  String get searchHint => 'Type words to search your Brain.';

  @override
  String get brainFolder => 'The Brain\'s folder';

  @override
  String notesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Notes',
      one: '1 Note',
      zero: 'No Notes',
    );
    return '$_temp0';
  }

  @override
  String version(String version) {
    return 'Version $version';
  }

  @override
  String errorIndex(String reason) {
    return 'The app\'s index could not be read or written: $reason';
  }

  @override
  String get takePhoto => 'Take a photo';

  @override
  String removeFile(String name) {
    return 'Remove $name';
  }

  @override
  String keptInInbox(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Captures were kept in your Inbox.',
      one: '1 Capture was kept in your Inbox.',
    );
    return '$_temp0';
  }

  @override
  String get share => 'Share';

  @override
  String get printCapture => 'Print Capture';

  @override
  String get printCaptureBody =>
      'Switch on IIbrain as a print service once. Then choose IIbrain as the printer in any app: the page arrives as a PDF in your Inbox the next time you open IIbrain.';

  @override
  String get openPrintSettings => 'Open the print settings';

  @override
  String get lookupCalendar => 'Calendar';

  @override
  String get lookupContacts => 'Contacts';

  @override
  String get lookupExplain =>
      'Read now, for this Question only. Nothing of it is kept in your Brain.';

  @override
  String get lookupRefused =>
      'IIbrain may not read this. Allow it in the system\'s settings for the app.';

  @override
  String get calendarToCloud => 'Calendar may go to Cloud Models';

  @override
  String get contactsToCloud => 'Contacts may go to Cloud Models';

  @override
  String get lookupLocalOnly => 'Off: it goes to Local Models only.';
}
