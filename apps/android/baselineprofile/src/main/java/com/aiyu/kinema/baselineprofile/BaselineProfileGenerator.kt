package com.aiyu.kinema.baselineprofile

import androidx.benchmark.macro.MacrobenchmarkScope
import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.StaleObjectException
import androidx.test.uiautomator.Until
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

private const val PACKAGE = BuildConfig.TARGET_APP_ID
private const val WAIT_MS = 5_000L

// Tab labels as the nav pills show them (strings.xml tab_*).
private val TABS = listOf("Home", "Library", "Search", "Settings")

@RunWith(AndroidJUnit4::class)
class BaselineProfileGenerator {
    @get:Rule
    val rule = BaselineProfileRule()

    // Cold start, then each tab; Home and Library also open a title. Signed in, it covers the shelves, grid and title
    // page. Signed out, the tabs are missing and only the Connect screen is covered.
    @Test
    fun startupAndScroll() = rule.collect(packageName = PACKAGE, includeInStartupProfile = true) {
        pressHome()
        startActivityAndWait()
        device.waitForIdle()
        for (tab in TABS) {
            device.wait(Until.findObject(By.text(tab)), WAIT_MS)?.click() ?: continue
            device.waitForIdle()
            scrollList()
            // Settings has no titles; its centre could be a sign-out button.
            if (tab == "Home" || tab == "Library") openTitleAndBack()
        }
    }
}

// The list is looked up again for every fling: images arriving recompose it and the old handle goes stale.
private fun MacrobenchmarkScope.scrollList() {
    for (direction in listOf(Direction.DOWN, Direction.DOWN, Direction.UP)) {
        try {
            val list = device.wait(Until.findObject(By.scrollable(true)), WAIT_MS) ?: return
            list.setGestureMargin(device.displayWidth / 5)
            list.fling(direction)
        } catch (_: StaleObjectException) {
            // Try the next direction on the fresh tree.
        }
        device.waitForIdle()
    }
}

// Opens the first card big enough to be a poster (not the nav, not a chip), then comes back. Pressing back on a screen
// that did not navigate would leave the app, so the app is started again if the tabs are gone.
private fun MacrobenchmarkScope.openTitleAndBack() {
    device.wait(Until.hasObject(By.clickable(true)), WAIT_MS)
    Thread.sleep(1_500)
    val card = device.findObjects(By.clickable(true)).firstOrNull {
        val b = it.visibleBounds
        b.width() > 200 && b.height() > 200 && b.left > device.displayWidth / 8
    } ?: return
    card.click()
    device.waitForIdle()
    Thread.sleep(1_500)
    device.pressBack()
    device.waitForIdle()
    if (!device.hasObject(By.text("Home"))) startActivityAndWait()
}
