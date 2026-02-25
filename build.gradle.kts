plugins {
    // this is necessary to avoid the plugins to be loaded multiple times
    // in each subproject's classloader
    alias(libs.plugins.composeHotReload) apply false
    alias(libs.plugins.composeMultiplatform) apply false
    alias(libs.plugins.composeCompiler) apply false
    alias(libs.plugins.kotlinMultiplatform) apply false
    alias(libs.plugins.sqldelight) apply false
    alias(libs.plugins.detekt) apply false
    alias(libs.plugins.kover) apply false
    alias(libs.plugins.versionsPlugin)
    alias(libs.plugins.dependencyLicenseReport)
}

licenseReport {
    outputDir = "$rootDir/build/reports/dependency-license"
    excludeOwnGroup = true
}

tasks.withType<com.github.benmanes.gradle.versions.updates.DependencyUpdatesTask> {
    rejectVersionIf {
        val v = candidate.version.lowercase()
        listOf("alpha", "beta", "rc", "cr", "m", "preview", "snapshot", "dev").any { v.contains(it) }
    }
}
