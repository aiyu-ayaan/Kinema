# Models are read by kotlinx.serialization's generated serializers.
-keepclassmembers @kotlinx.serialization.Serializable class com.aiyu.kinema.core.** {
    *** Companion;
    kotlinx.serialization.KSerializer serializer(...);
}
