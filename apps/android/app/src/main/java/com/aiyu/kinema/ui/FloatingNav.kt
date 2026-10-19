package com.aiyu.kinema.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.displayCutoutPadding
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.ui.theme.rememberReducedMotion

data class NavTab(val label: String, val icon: ImageVector, val selectedIcon: ImageVector)

/** Space the floating pill covers at the bottom; screens pad their last item by it. Zero beside the rail. */
val LocalBottomBarInset = compositionLocalOf { 0.dp }

/**
 * Tab host: a floating pill on phones (tucks away while scrolling down, the selected
 * tab grows into icon + label), the same pill stood upright as a rail from 600dp. Tabs switch with a fade-through.
 */
@Composable
fun FloatingNavScaffold(tabs: List<NavTab>, selected: Int, onSelect: (Int) -> Unit, content: @Composable (Int) -> Unit) {
    val motion = MaterialTheme.motionScheme
    val effects = motion.defaultEffectsSpec<Float>()
    val fastEffects = motion.fastEffectsSpec<Float>()
    val spatial = motion.defaultSpatialSpec<Float>()
    // A tab keeps its scroll while another one is shown.
    val saved = rememberSaveableStateHolder()
    val page: @Composable () -> Unit = {
        AnimatedContent(
            targetState = selected,
            modifier = Modifier.fillMaxSize(),
            // M3 "fade through": outgoing fades fast, incoming fades + scales up slightly.
            transitionSpec = { (fadeIn(effects) + scaleIn(spatial, initialScale = 0.94f)).togetherWith(fadeOut(fastEffects)) },
            label = "tab",
        ) { current -> saved.SaveableStateProvider(current) { Surface(Modifier.fillMaxSize()) { content(current) } } }
    }

    BoxWithConstraints(Modifier.fillMaxSize()) {
        if (maxWidth >= 600.dp) {
            Row(Modifier.fillMaxSize()) {
                FloatingNavRail(tabs, selected, onSelect)
                Box(Modifier.weight(1f)) { page() }
            }
        } else {
            // The pill floats over content (no reserved strip). Only real scrolls count, so a screen that can't
            // scroll keeps it.
            var scrolledAway by remember { mutableStateOf(false) }
            LaunchedEffect(selected) { scrolledAway = false }
            val hideOnScroll = remember {
                object : NestedScrollConnection {
                    override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset {
                        if (consumed.y < -1f) scrolledAway = true else if (consumed.y > 1f) scrolledAway = false
                        return Offset.Zero
                    }
                }
            }
            val density = LocalDensity.current
            // Saved in pixels: a screen restored at its very end would otherwise be measured without the inset and pushed up.
            var pill by rememberSaveable { mutableIntStateOf(0) }
            Box(Modifier.fillMaxSize().nestedScroll(hideOnScroll)) {
                CompositionLocalProvider(LocalBottomBarInset provides with(density) { pill.toDp() }) { page() }
                AnimatedVisibility(
                    !scrolledAway,
                    modifier = Modifier.align(Alignment.BottomCenter),
                    enter = slideInVertically(motion.defaultSpatialSpec<IntOffset>()) { it } + fadeIn(effects),
                    exit = slideOutVertically(motion.defaultSpatialSpec<IntOffset>()) { it } + fadeOut(fastEffects),
                ) {
                    FloatingNavBar(tabs, selected, onSelect, Modifier.onSizeChanged { pill = it.height })
                }
            }
        }
    }
}

@Composable
private fun FloatingNavBar(tabs: List<NavTab>, selected: Int, onSelect: (Int) -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxWidth().navigationBarsPadding().padding(bottom = 12.dp), contentAlignment = Alignment.Center) {
        Surface(shape = CircleShape, color = MaterialTheme.colorScheme.surfaceContainerHigh, shadowElevation = 6.dp) {
            Row(Modifier.padding(6.dp).selectableGroup(), horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                tabs.forEachIndexed { index, tab -> PillItem(tab, index == selected) { onSelect(index) } }
            }
        }
    }
}

@Composable
private fun PillItem(tab: NavTab, selected: Boolean, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    val effects = MaterialTheme.motionScheme.defaultEffectsSpec<Color>()
    val bg by animateColorAsState(if (selected) c.primaryContainer else Color.Transparent, effects, label = "pillBg")
    val fg by animateColorAsState(if (selected) c.onPrimaryContainer else c.onSurfaceVariant, effects, label = "pillFg")
    Row(
        Modifier
            .clip(CircleShape)
            .background(bg)
            .selectable(selected, onClick = onClick, role = Role.Tab)
            .semantics(mergeDescendants = true) { contentDescription = tab.label }
            .heightIn(min = 48.dp)
            .padding(horizontal = 16.dp)
            .animateContentSize(MaterialTheme.motionScheme.defaultSpatialSpec()),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        NavIcon(tab, selected, tint = fg)
        if (selected) {
            Spacer(Modifier.width(8.dp))
            Text(tab.label, style = MaterialTheme.typography.labelLarge, color = fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.widthIn(max = 120.dp))
        }
    }
}

/** Tab icon that pops in with a small bouncy twist when its tab gets selected (not on first show, not with reduced motion). */
@Composable
private fun NavIcon(tab: NavTab, selected: Boolean, tint: Color = LocalContentColor.current) {
    val reduced = rememberReducedMotion()
    val pop = remember { Animatable(1f) }
    var shown by remember { mutableStateOf(false) }
    LaunchedEffect(selected) {
        if (selected && shown && !reduced) {
            pop.snapTo(0.6f)
            pop.animateTo(1f, spring(dampingRatio = Spring.DampingRatioHighBouncy, stiffness = Spring.StiffnessMediumLow))
        } else {
            pop.snapTo(1f) // a fast tab switch cancels the pop mid-way; never leave the icon shrunk
        }
        shown = true
    }
    Icon(
        if (selected) tab.selectedIcon else tab.icon,
        contentDescription = null,
        tint = tint,
        modifier = Modifier.graphicsLayer {
            scaleX = pop.value
            scaleY = pop.value
            rotationZ = (1f - pop.value) * -40f
        },
    )
}

@Composable
private fun FloatingNavRail(tabs: List<NavTab>, selected: Int, onSelect: (Int) -> Unit) {
    Box(Modifier.fillMaxHeight().systemBarsPadding().displayCutoutPadding().padding(start = 12.dp, top = 12.dp, bottom = 12.dp), contentAlignment = Alignment.Center) {
        Surface(shape = MaterialTheme.shapes.extraLarge, color = MaterialTheme.colorScheme.surfaceContainerHigh, shadowElevation = 6.dp) {
            Column(
                Modifier.width(88.dp).padding(vertical = 12.dp).verticalScroll(rememberScrollState()).selectableGroup(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                tabs.forEachIndexed { index, tab ->
                    NavigationRailItem(
                        selected = index == selected,
                        onClick = { onSelect(index) },
                        icon = { NavIcon(tab, index == selected) },
                        label = { Text(tab.label, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                        colors = NavigationRailItemDefaults.colors(
                            indicatorColor = MaterialTheme.colorScheme.primaryContainer,
                            selectedIconColor = MaterialTheme.colorScheme.onPrimaryContainer,
                        ),
                    )
                }
            }
        }
    }
}

/** True where a screen has room for more than a phone's layout: a tab from 840dp wide. False again inside one of its [SideBySide] columns. */
val LocalWideScreen = compositionLocalOf { false }

/**
 * Scrolling body of a tab: clear of the status bar and the floating pill, 16dp gutters. Capped at a readable
 * 840dp, or 1240dp on a wide window, where it sets [LocalWideScreen] so screens spread into columns.
 */
@Composable
fun ScreenColumn(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val bottom: Dp = LocalBottomBarInset.current
    BoxWithConstraints(modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
        val wide = maxWidth >= 840.dp
        CompositionLocalProvider(LocalWideScreen provides wide) {
            Column(
                Modifier
                    .widthIn(max = if (wide) 1240.dp else 840.dp)
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .statusBarsPadding()
                    .padding(horizontal = 16.dp)
                    .padding(bottom = bottom + 24.dp)
                    .navigationBarsPadding(),
                verticalArrangement = Arrangement.spacedBy(16.dp),
                content = content,
            )
        }
    }
}
