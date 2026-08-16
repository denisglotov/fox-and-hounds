# Optimization configuration
-repackageclasses ''
-allowaccessmodification

# Keep Android Activity and entry point
-keep public class org.dymka.foxandhounds.MainActivity {
    public *;
}

# Keep JNI bindings for miniquad / Macroquad
-keep public class quad_native.QuadNative {
    public static native *;
    public *;
}

# Keep all native methods across the project
-keepclasseswithmembernames class * {
    native <methods>;
}

# Preserve debugging and stacktrace information for de-obfuscation
-keepattributes SourceFile,LineNumberTable,*Annotation*,Signature,InnerClasses,EnclosingMethod

# Suppress harmless warnings from framework references
-dontwarn **
