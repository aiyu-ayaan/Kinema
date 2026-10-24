package com.aiyu.kinema.baselineprofile

import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

private const val PACKAGE = BuildConfig.TARGET_APP_ID

@RunWith(AndroidJUnit4::class)
class BaselineProfileGenerator {
    @get:Rule
    val rule = BaselineProfileRule()

    // Cold start, then scroll whatever list the first screen shows (library grid when signed in).
    @Test
    fun startupAndScroll() = rule.collect(packageName = PACKAGE, includeInStartupProfile = true) {
        pressHome()
        startActivityAndWait()
        device.waitForIdle()
        device.findObject(By.scrollable(true))?.let { list ->
            list.setGestureMargin(device.displayWidth / 5)
            repeat(3) { list.fling(Direction.DOWN) }
            device.waitForIdle()
            repeat(2) { list.fling(Direction.UP) }
        }
    }
}
