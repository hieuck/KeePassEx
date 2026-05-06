# KeePassEx Android ProGuard rules

# React Native
-keep class com.facebook.react.** { *; }
-keep class com.facebook.hermes.** { *; }
-keep class com.facebook.jni.** { *; }

# KeePassEx native module
-keep class com.keepassex.KeePassExCoreModule { *; }
-keep class com.keepassex.KeePassExCorePackage { *; }
-keep class com.keepassex.autofill.** { *; }
-keep class com.keepassex.quicktile.** { *; }

# Rust JNI
-keep class com.keepassex.** { native <methods>; }

# Biometrics
-keep class androidx.biometric.** { *; }

# AutoFill
-keep class android.service.autofill.** { *; }
-keep class android.view.autofill.** { *; }

# Keep all annotations
-keepattributes *Annotation*
-keepattributes Signature
-keepattributes Exceptions

# Kotlin
-keep class kotlin.** { *; }
-keep class kotlinx.coroutines.** { *; }
-dontwarn kotlin.**
