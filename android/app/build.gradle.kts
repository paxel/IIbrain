import javax.inject.Inject

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "io.github.paxel.iibrain"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "io.github.paxel.iibrain"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
}

// The core (docs/adr/0008, 0011): the Rust workspace in desktop/, cross-built
// into libiibrain.so per ABI. A debug build is for a 64-bit phone only,
// which halves the native build.
androidComponents {
    onVariants { variant ->
        val workspace = rootProject.layout.projectDirectory.dir("../desktop")
        val task = tasks.register<CargoNdk>(
            "cargoNdk" + variant.name.replaceFirstChar { it.uppercase() },
        ) {
            crate.set(workspace)
            sources.from(
                workspace.dir("crates/iibrain-core/src"),
                workspace.dir("crates/iibrain-core/models"),
                workspace.dir("crates/iibrain-ffi/src"),
                workspace.file("crates/iibrain-core/Cargo.toml"),
                workspace.file("crates/iibrain-ffi/Cargo.toml"),
                workspace.file("Cargo.toml"),
                workspace.file("Cargo.lock"),
            )
            abis.set(
                if (variant.buildType == "debug") {
                    listOf("arm64-v8a")
                } else {
                    listOf("arm64-v8a", "armeabi-v7a")
                },
            )
            platform.set(flutter.minSdkVersion)
            ndk.set(androidComponents.sdkComponents.ndkDirectory.map { it.asFile.absolutePath })
            targetDir.set(layout.buildDirectory.dir("core/target"))
        }
        variant.sources.jniLibs?.addGeneratedSourceDirectory(task, CargoNdk::outputDir)
    }
}

/**
 * Cross-builds the core with cargo-ndk into libiibrain.so per ABI. Always
 * optimised, in debug builds too: the text recognition is far too slow
 * otherwise. At most two compile jobs, so the build never takes the machine
 * down. Up to date, and not run, while the core's sources and lock file
 * stay the same; cargo's own target directory keeps the rest incremental.
 */
abstract class CargoNdk @Inject constructor(
    private val exec: ExecOperations,
    private val files: FileSystemOperations,
) : DefaultTask() {
    @get:Internal
    abstract val crate: DirectoryProperty

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sources: ConfigurableFileCollection

    @get:Input
    abstract val abis: ListProperty<String>

    @get:Input
    abstract val platform: Property<Int>

    @get:Input
    abstract val ndk: Property<String>

    @get:Internal
    abstract val targetDir: DirectoryProperty

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @TaskAction
    fun build() {
        val target = targetDir.get().asFile
        exec.exec {
            workingDir = crate.get().asFile
            environment("ANDROID_NDK_HOME", ndk.get())
            environment("CARGO_TARGET_DIR", target.absolutePath)
            commandLine(
                listOf("cargo", "ndk", "--platform", platform.get().toString()) +
                    abis.get().flatMap { listOf("--target", it) } +
                    listOf("build", "-p", "iibrain-ffi", "--release", "--locked", "-j", "2"),
            )
        }
        files.sync {
            for (abi in abis.get()) {
                from(File(target, "${TRIPLES.getValue(abi)}/release/libiibrain.so")) {
                    into(abi)
                }
            }
            into(outputDir)
        }
    }

    companion object {
        /** The Rust target of each Android ABI built. */
        val TRIPLES = mapOf(
            "arm64-v8a" to "aarch64-linux-android",
            "armeabi-v7a" to "armv7-linux-androideabi",
        )
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
