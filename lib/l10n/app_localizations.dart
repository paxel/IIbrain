import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_de.dart';
import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('de'),
    Locale('en'),
  ];

  /// The app's name, never translated.
  ///
  /// In en, this message translates to:
  /// **'IIbrain'**
  String get appTitle;

  /// Heading of the first start, before a folder is chosen.
  ///
  /// In en, this message translates to:
  /// **'Where is your Brain?'**
  String get chooseBrainTitle;

  /// Explains the Brain's folder on the first start.
  ///
  /// In en, this message translates to:
  /// **'Your Brain is a folder of Markdown files. Choose an existing folder or an empty one. IIbrain never syncs it; a sync tool of your choice can carry it between your devices.'**
  String get chooseBrainBody;

  /// Button that opens the folder picker for the Brain.
  ///
  /// In en, this message translates to:
  /// **'Choose folder…'**
  String get chooseFolder;

  /// Button that points the app at another folder as the Brain.
  ///
  /// In en, this message translates to:
  /// **'Change folder…'**
  String get changeFolder;

  /// Shown when the chosen folder cannot be used as the Brain.
  ///
  /// In en, this message translates to:
  /// **'This folder could not be opened: {reason}'**
  String folderNotOpened(String reason);

  /// Heading over the list of Folders.
  ///
  /// In en, this message translates to:
  /// **'Folders'**
  String get folders;

  /// The one fixed Folder, where Unfiled Notes wait.
  ///
  /// In en, this message translates to:
  /// **'Inbox'**
  String get inbox;

  /// Name shown for Notes lying directly in the Brain's folder, outside every Folder.
  ///
  /// In en, this message translates to:
  /// **'Brain'**
  String get brainRoot;

  /// Shown for a Folder without Notes.
  ///
  /// In en, this message translates to:
  /// **'No Notes here.'**
  String get noNotesHere;

  /// Shown where a Note opens, while none is open.
  ///
  /// In en, this message translates to:
  /// **'Choose a Note to read it.'**
  String get chooseNote;

  /// Heading over a Note's Attachments.
  ///
  /// In en, this message translates to:
  /// **'Attachments'**
  String get attachments;

  /// Heading over the text the app read out of an Attachment.
  ///
  /// In en, this message translates to:
  /// **'Extracted Text'**
  String get extractedText;

  /// When a Note was captured, under its title.
  ///
  /// In en, this message translates to:
  /// **'Captured {when}'**
  String capturedOn(String when);

  /// Button and dialog title for a typed Capture.
  ///
  /// In en, this message translates to:
  /// **'New Note'**
  String get newNote;

  /// Label of the text field of a Note.
  ///
  /// In en, this message translates to:
  /// **'Text'**
  String get noteText;

  /// Saves what a dialog holds.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// Closes a dialog without saving.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// Hint in the desktop's capture dialog about what only the phone has.
  ///
  /// In en, this message translates to:
  /// **'Print Capture, the camera and Live Lookups are found on the phone.'**
  String get phoneOnly;

  /// Button that opens a Note's text for editing.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get edit;

  /// Button that files an Unfiled Note by hand.
  ///
  /// In en, this message translates to:
  /// **'File…'**
  String get fileNote;

  /// Title of the dialog that files an Unfiled Note.
  ///
  /// In en, this message translates to:
  /// **'File this Note'**
  String get fileNoteTitle;

  /// Button that changes title, tags and Folder of a filed Note.
  ///
  /// In en, this message translates to:
  /// **'Title, tags and Folder…'**
  String get noteDetails;

  /// Title of the dialog that changes a filed Note.
  ///
  /// In en, this message translates to:
  /// **'Title, tags and Folder'**
  String get noteDetailsTitle;

  /// Button that deletes a Note.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get delete;

  /// Asks before a Note is deleted.
  ///
  /// In en, this message translates to:
  /// **'Delete this Note with its Attachments?'**
  String get deleteNoteQuestion;

  /// Label of a Note's title field.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get titleLabel;

  /// Label of a Note's tags field.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get tagsLabel;

  /// Hint in the empty tags field.
  ///
  /// In en, this message translates to:
  /// **'separated by commas'**
  String get tagsHint;

  /// Label of the choice of a Note's Folder.
  ///
  /// In en, this message translates to:
  /// **'Folder'**
  String get folderLabel;

  /// Button that makes a Folder.
  ///
  /// In en, this message translates to:
  /// **'New Folder…'**
  String get newFolder;

  /// Title of the dialog that makes a Folder.
  ///
  /// In en, this message translates to:
  /// **'New Folder'**
  String get newFolderTitle;

  /// Button that renames the chosen Folder.
  ///
  /// In en, this message translates to:
  /// **'Rename…'**
  String get renameFolder;

  /// Title of the dialog that renames a Folder.
  ///
  /// In en, this message translates to:
  /// **'Rename Folder'**
  String get renameFolderTitle;

  /// Button that removes the chosen, empty Folder.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get removeFolder;

  /// Label of a Folder's name field.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get folderName;

  /// Confirms making a Folder.
  ///
  /// In en, this message translates to:
  /// **'Create'**
  String get create;

  /// Confirms renaming a Folder.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get rename;

  /// Over the Inbox: how many Notes still wait to be filed.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Nothing waits Unfiled.} =1{1 Note waits Unfiled.} other{{count} Notes wait Unfiled.}}'**
  String unfiledWaiting(int count);

  /// A request the app cannot follow, such as a name that is taken or a Folder that still holds Notes.
  ///
  /// In en, this message translates to:
  /// **'That is not possible: {reason}'**
  String errorNotPossible(String reason);

  /// A Note, Folder or file that is gone.
  ///
  /// In en, this message translates to:
  /// **'Not found: {what}'**
  String errorNotFound(String what);

  /// Reading or writing the Brain's folder failed.
  ///
  /// In en, this message translates to:
  /// **'A file could not be read or written: {reason}'**
  String errorFile(String reason);

  /// Any other failure.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong: {reason}'**
  String errorOther(String reason);

  /// Label of the search field.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get search;

  /// Over the search results: how many Notes hold the words.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Nothing found.} =1{1 Note found.} other{{count} Notes found.}}'**
  String searchResults(int count);

  /// Button that empties the search field and shows the Folder again.
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clearSearch;

  /// Button in the capture dialog that picks files to keep as Attachments.
  ///
  /// In en, this message translates to:
  /// **'Add files…'**
  String get addFiles;

  /// Under an Attachment the app read without finding text.
  ///
  /// In en, this message translates to:
  /// **'No text was found in it.'**
  String get noTextFound;

  /// Under an Attachment whose text the app has not read yet.
  ///
  /// In en, this message translates to:
  /// **'Its text is being read…'**
  String get readingText;

  /// Heading over the Extracted Text of the page behind a Note's link.
  ///
  /// In en, this message translates to:
  /// **'Text of the page'**
  String get pageText;

  /// Under a link whose page the app could not fetch.
  ///
  /// In en, this message translates to:
  /// **'The text of the page is missing: it could not be fetched.'**
  String get pageTextMissing;

  /// Under a link whose page the app has not fetched yet.
  ///
  /// In en, this message translates to:
  /// **'The page is being fetched…'**
  String get pageFetching;

  /// Hint in the capture dialog.
  ///
  /// In en, this message translates to:
  /// **'A link on its own is kept together with the text of its page.'**
  String get linkHint;

  /// Button that opens the list of connected Models.
  ///
  /// In en, this message translates to:
  /// **'Models…'**
  String get models;

  /// Title of the dialog with the connected Models.
  ///
  /// In en, this message translates to:
  /// **'Models'**
  String get modelsTitle;

  /// In the Models dialog while it is empty.
  ///
  /// In en, this message translates to:
  /// **'No Model is connected. IIbrain brings none of its own; without one you capture, file by hand and search.'**
  String get noModels;

  /// Beside a Model the person declared as running under their own control.
  ///
  /// In en, this message translates to:
  /// **'Local Model'**
  String get localModel;

  /// Beside a Model the person declared as run by someone else.
  ///
  /// In en, this message translates to:
  /// **'Cloud Model'**
  String get cloudModel;

  /// Button that removes a Model from this Device.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get remove;

  /// Heading over the fields for a new Model.
  ///
  /// In en, this message translates to:
  /// **'Connect a Model'**
  String get connectModel;

  /// Label of the field with what the person calls a Model.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get modelName;

  /// Label of the field with the endpoint's base address.
  ///
  /// In en, this message translates to:
  /// **'Address'**
  String get modelAddress;

  /// Hint in the empty address field.
  ///
  /// In en, this message translates to:
  /// **'such as http://localhost:11434/v1'**
  String get modelAddressHint;

  /// Label of the field with the endpoint's key.
  ///
  /// In en, this message translates to:
  /// **'Key'**
  String get modelKey;

  /// Label of the field with the model's name at the endpoint.
  ///
  /// In en, this message translates to:
  /// **'Model at the endpoint'**
  String get modelId;

  /// Checkbox that declares a new Model local.
  ///
  /// In en, this message translates to:
  /// **'It runs under my own control (Local Model)'**
  String get modelIsLocal;

  /// Under the fields for a new Model.
  ///
  /// In en, this message translates to:
  /// **'Address and key stay on this device and never enter the Brain\'s folder.'**
  String get modelStaysHere;

  /// Button that tries a new Model's connection and adds it when it answers.
  ///
  /// In en, this message translates to:
  /// **'Test and add'**
  String get testAndAdd;

  /// Button that adds a new Model without trying its connection.
  ///
  /// In en, this message translates to:
  /// **'Add without testing'**
  String get addUntested;

  /// Closes a dialog that saves as you go.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// A Model was unreachable, refused, or answered in no known form.
  ///
  /// In en, this message translates to:
  /// **'The Model did not answer: {reason}'**
  String errorModel(String reason);

  /// The mark on a Note or Folder that keeps it from Cloud Models; also the checkbox that sets a Note's own mark.
  ///
  /// In en, this message translates to:
  /// **'Private'**
  String get privateMark;

  /// Explains the Private mark where it is set.
  ///
  /// In en, this message translates to:
  /// **'What is Private reaches only a Local Model, never a Cloud Model.'**
  String get privateExplain;

  /// Under a Note that is Private through its Folder's mark.
  ///
  /// In en, this message translates to:
  /// **'Private because its Folder is.'**
  String get privateByFolder;

  /// Under an Unfiled Note while Unfiled Notes count as Private.
  ///
  /// In en, this message translates to:
  /// **'Private as long as it is Unfiled.'**
  String get privateUnfiled;

  /// Button that sets the Private mark of the chosen Folder.
  ///
  /// In en, this message translates to:
  /// **'Mark Private'**
  String get folderMarkPrivate;

  /// Button that lifts the Private mark of the chosen Folder.
  ///
  /// In en, this message translates to:
  /// **'Lift Private mark'**
  String get folderLiftPrivate;

  /// Checkbox over the Inbox: whether fresh Captures are Private until filed.
  ///
  /// In en, this message translates to:
  /// **'Unfiled Notes count as Private'**
  String get unfiledPrivateSetting;

  /// A Folder's or Note's name in a list when it is Private.
  ///
  /// In en, this message translates to:
  /// **'{name} · Private'**
  String privateSuffix(String name);

  /// Button in the filing dialog that asks the Model for a Proposal.
  ///
  /// In en, this message translates to:
  /// **'Propose'**
  String get propose;

  /// While the app waits for a Model's answer.
  ///
  /// In en, this message translates to:
  /// **'The Model is working…'**
  String get modelWorking;

  /// Over the filing fields once a Proposal filled them.
  ///
  /// In en, this message translates to:
  /// **'Proposal by {model}. Correct it or save it as it is.'**
  String proposalBy(String model);

  /// A Proposal or an Answer was asked for while no Model is connected.
  ///
  /// In en, this message translates to:
  /// **'No Model is connected. Connect one under Models…, or do this by hand.'**
  String get errorNoModel;

  /// The privacy gate refused to send something Private to a Cloud Model.
  ///
  /// In en, this message translates to:
  /// **'This is Private and your Model is a Cloud Model, so nothing was sent. Choose a Local Model, or lift the Private mark.'**
  String get errorPrivate;

  /// A Folder a Proposal suggests that does not exist yet, in the Folder choice.
  ///
  /// In en, this message translates to:
  /// **'{name} (new Folder)'**
  String newFolderSuffix(String name);

  /// Button that opens the ask screen, and the button that sends a Question.
  ///
  /// In en, this message translates to:
  /// **'Ask'**
  String get ask;

  /// Heading of the ask screen.
  ///
  /// In en, this message translates to:
  /// **'Ask your Brain'**
  String get askTitle;

  /// Label of the field for a Question.
  ///
  /// In en, this message translates to:
  /// **'Question'**
  String get questionLabel;

  /// Button that leaves the ask screen.
  ///
  /// In en, this message translates to:
  /// **'Back to the Notes'**
  String get backToNotes;

  /// On the ask screen when the search found no Note for a Question.
  ///
  /// In en, this message translates to:
  /// **'Nothing in your Brain fits this Question. No Model was asked.'**
  String get nothingFits;

  /// Over an Answer.
  ///
  /// In en, this message translates to:
  /// **'Answer by {model}, from the Sources below alone:'**
  String answerBy(String model);

  /// Heading over the Notes an Answer was made from.
  ///
  /// In en, this message translates to:
  /// **'Sources'**
  String get sources;

  /// On the ask screen when Private Notes fitted the Question but were not sent to a Cloud Model.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 Private Note was left out, because your Model is a Cloud Model. The Answer may be incomplete.} other{{count} Private Notes were left out, because your Model is a Cloud Model. The Answer may be incomplete.}}'**
  String privateLeftOut(int count);

  /// Button under an Answer that makes it a Saved Answer.
  ///
  /// In en, this message translates to:
  /// **'Keep as a Note'**
  String get keepAnswer;

  /// On a Saved Answer, over the links to its Sources.
  ///
  /// In en, this message translates to:
  /// **'Written by the Model {model}, from these Sources:'**
  String writtenByModel(String model);

  /// A Saved Answer's title in a list.
  ///
  /// In en, this message translates to:
  /// **'{name} · Answer'**
  String answerSuffix(String name);

  /// A Source of a Saved Answer whose Note was moved or deleted.
  ///
  /// In en, this message translates to:
  /// **'{name} (no longer there)'**
  String sourceGone(String name);

  /// A Note's title in a list while it is in Conflict.
  ///
  /// In en, this message translates to:
  /// **'{name} · Conflict'**
  String conflictSuffix(String name);

  /// Over a Note in Conflict.
  ///
  /// In en, this message translates to:
  /// **'{count} versions of this Note lie side by side: two devices changed it and your sync tool kept both.'**
  String conflictBanner(int count);

  /// Button that opens the versions of a Note in Conflict side by side.
  ///
  /// In en, this message translates to:
  /// **'Resolve…'**
  String get resolveConflict;

  /// Title of the dialog that shows the versions of a Note in Conflict.
  ///
  /// In en, this message translates to:
  /// **'Conflict'**
  String get conflictTitle;

  /// Heading over one version of a Note in Conflict.
  ///
  /// In en, this message translates to:
  /// **'Version {number}'**
  String versionOf(int number);

  /// Button that ends a Conflict by keeping one version.
  ///
  /// In en, this message translates to:
  /// **'Keep version {number}'**
  String keepVersion(int number);

  /// Label of the field in which the person merges the versions of a Note.
  ///
  /// In en, this message translates to:
  /// **'Merged text'**
  String get mergedText;

  /// Button that ends a Conflict with the merged text.
  ///
  /// In en, this message translates to:
  /// **'Keep the merged text'**
  String get keepMerged;

  /// Heading on the phone before all-files access is granted.
  ///
  /// In en, this message translates to:
  /// **'IIbrain needs access to your files'**
  String get allowFilesTitle;

  /// Explains on the phone why all-files access is asked for.
  ///
  /// In en, this message translates to:
  /// **'Your Brain is a folder in the phone\'s storage that your sync tool also writes to. To read and write it, IIbrain needs the permission to manage all files. It uses it for that folder only.'**
  String get allowFilesBody;

  /// Button on the phone that opens the system's page for all-files access.
  ///
  /// In en, this message translates to:
  /// **'Open the permission'**
  String get allowFilesButton;

  /// Button in the phone's folder browser that makes the shown folder the Brain.
  ///
  /// In en, this message translates to:
  /// **'Use this folder'**
  String get useThisFolder;

  /// Goes to the parent folder in the phone's folder browser.
  ///
  /// In en, this message translates to:
  /// **'Up'**
  String get folderUp;

  /// In the phone's folder browser for a folder without sub-folders.
  ///
  /// In en, this message translates to:
  /// **'No folders in here.'**
  String get noFoldersHere;

  /// The phone's tab with the Folders and Notes.
  ///
  /// In en, this message translates to:
  /// **'Brain'**
  String get tabBrain;

  /// The phone's search tab.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get tabSearch;

  /// The phone's settings tab.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get tabSettings;

  /// On the phone's search tab while the field is empty.
  ///
  /// In en, this message translates to:
  /// **'Type words to search your Brain.'**
  String get searchHint;

  /// Label of the setting that shows where the Brain lies.
  ///
  /// In en, this message translates to:
  /// **'The Brain\'s folder'**
  String get brainFolder;

  /// Under a Folder's name in the phone's list.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{No Notes} =1{1 Note} other{{count} Notes}}'**
  String notesCount(int count);

  /// The app's version in the settings.
  ///
  /// In en, this message translates to:
  /// **'Version {version}'**
  String version(String version);

  /// The index database on the Device failed.
  ///
  /// In en, this message translates to:
  /// **'The app\'s index could not be read or written: {reason}'**
  String errorIndex(String reason);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['de', 'en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'de':
      return AppLocalizationsDe();
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
