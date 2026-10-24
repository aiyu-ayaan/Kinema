package com.aiyu.kinema.baselineprofile

import androidx.benchmark.macro.BaselineProfileMode
import androidx.benchmark.macro.CompilationMode
import androidx.benchmark.macro.StartupMode
import androidx.benchmark.macro.StartupTimingMetric
import androidx.benchmark.macro.junit4.MacrobenchmarkRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

private const val PACKAGE = BuildConfig.TARGET_APP_ID

// Compare: ./gradlew :baselineprofile:connectedProdBenchmarkAndroidTest
@RunWith(AndroidJUnit4::class)
class StartupBenchmark {
    @get:Rule
    val rule = MacrobenchmarkRule()

    @Test
    fun coldStartNoCompilation() = start(CompilationMode.None())

    @Test
    fun coldStartBaselineProfile() = start(CompilationMode.Partial(BaselineProfileMode.Require))

    private fun start(mode: CompilationMode) = rule.measureRepeated(
        packageName = PACKAGE,
        metrics = listOf(StartupTimingMetric()),
        compilationMode = mode,
        startupMode = StartupMode.COLD,
        iterations = 8,
    ) {
        pressHome()
        startActivityAndWait()
    }
}
