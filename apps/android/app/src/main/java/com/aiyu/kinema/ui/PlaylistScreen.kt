package com.aiyu.kinema.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.DeleteOutline
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.PlaylistPlay
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.R
import com.aiyu.kinema.core.ItemCard
import com.aiyu.kinema.core.PlaylistDetail
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlaylistScreen(id: Long) {
    val nav = LocalNavigator.current
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    val loader = rememberLoad(id) { it.playlist(id) }
    val snackbar = remember { SnackbarHostState() }
    var showDeleteConfirm by remember { mutableStateOf(false) }

    fun deletePlaylist() {
        scope.launch {
            repository.request { it.deletePlaylist(id) }
                .onSuccess {
                    repository.notifyChanged()
                    nav.back()
                }
                .onFailure {
                    snackbar.showSnackbar("Failed to delete playlist")
                }
        }
    }

    fun removeItem(item: ItemCard, detail: PlaylistDetail) {
        val updatedIds = detail.items.filter { it.id != item.id }.map { it.id }
        scope.launch {
            repository.request { it.setPlaylistItems(id, updatedIds) }
                .onSuccess {
                    repository.notifyChanged()
                    loader.reload()
                }
                .onFailure {
                    snackbar.showSnackbar("Failed to remove item")
                }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { },
                navigationIcon = {
                    IconButton(onClick = { nav.back() }) {
                        Icon(Icons.AutoMirrored.Rounded.ArrowBack, stringResource(R.string.action_back))
                    }
                },
                actions = {
                    IconButton(onClick = { showDeleteConfirm = true }) {
                        Icon(Icons.Rounded.DeleteOutline, stringResource(R.string.playlist_delete))
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
                modifier = Modifier.statusBarsPadding(),
            )
        },
        snackbarHost = { SnackbarHost(snackbar, Modifier.navigationBarsPadding()) },
    ) { padding ->
        Loaded(loader) { detail ->
            if (showDeleteConfirm) {
                AlertDialog(
                    onDismissRequest = { showDeleteConfirm = false },
                    title = { Text(stringResource(R.string.playlist_delete)) },
                    text = { Text(stringResource(R.string.playlist_delete_confirm, detail.name)) },
                    confirmButton = {
                        TextButton(
                            onClick = {
                                showDeleteConfirm = false
                                deletePlaylist()
                            },
                        ) {
                            Text(stringResource(R.string.action_delete), color = MaterialTheme.colorScheme.error)
                        }
                    },
                    dismissButton = {
                        TextButton(onClick = { showDeleteConfirm = false }) {
                            Text(stringResource(R.string.action_cancel))
                        }
                    },
                )
            }

            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
                contentPadding = PaddingValues(bottom = 32.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                item(key = "header") {
                    Column(
                        Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 20.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(16.dp),
                        ) {
                            Box(
                                Modifier
                                    .size(72.dp)
                                    .clip(MaterialTheme.shapes.large)
                                    .background(MaterialTheme.colorScheme.primaryContainer),
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    Icons.Rounded.PlaylistPlay,
                                    null,
                                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                    modifier = Modifier.size(36.dp),
                                )
                            }
                            Column(Modifier.weight(1f)) {
                                Text(
                                    detail.name,
                                    style = MaterialTheme.typography.headlineMedium,
                                    maxLines = 2,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                Text(
                                    pluralStringResource(
                                        R.plurals.playlist_items_count,
                                        detail.items.size,
                                        detail.items.size,
                                    ),
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }

                        if (detail.items.isNotEmpty()) {
                            Button(
                                onClick = {
                                    val first = detail.items.first()
                                    nav.play(first.id, null)
                                },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(52.dp),
                                shape = CircleShape,
                            ) {
                                Icon(Icons.Rounded.PlayArrow, null)
                                Spacer(Modifier.width(8.dp))
                                Text(stringResource(R.string.playlist_play_all), style = MaterialTheme.typography.titleMedium)
                            }
                        }
                    }
                }

                if (detail.items.isEmpty()) {
                    item(key = "empty") {
                        MessageState(
                            icon = Icons.Rounded.PlaylistPlay,
                            title = stringResource(R.string.playlist_empty),
                            message = stringResource(R.string.playlist_empty_body),
                        )
                    }
                } else {
                    itemsIndexed(detail.items, key = { _, item -> item.id }) { index, item ->
                        PlaylistItemRow(
                            index = index + 1,
                            item = item,
                            onClick = { nav.open(item.id) },
                            onRemove = { removeItem(item, detail) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun PlaylistItemRow(
    index: Int,
    item: ItemCard,
    onClick: () -> Unit,
    onRemove: () -> Unit,
) {
    val api = LocalApi.current
    val title = if (item.kind == "episode") item.showTitle ?: item.title else item.title
    val subtitle = if (item.kind == "episode") listOfNotNull(subtitleOf(item), item.title).joinToString(" · ") else subtitleOf(item)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Text(
            text = "$index",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.width(20.dp),
        )

        Box(
            Modifier
                .width(64.dp)
                .aspectRatio(16f / 9f)
                .clip(MaterialTheme.shapes.small),
        ) {
            val key = item.backdrop ?: item.poster
            Artwork(
                image = api?.imageUrl(key),
                frame = api?.frameUrl(item.id),
                title = title,
                poster = false,
                modifier = Modifier.fillMaxSize(),
            )
        }

        Column(Modifier.weight(1f)) {
            Text(
                title,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (subtitle != null) {
                Text(
                    subtitle,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }

        IconButton(onClick = onRemove) {
            Icon(
                Icons.Rounded.Delete,
                stringResource(R.string.playlist_remove),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
                modifier = Modifier.size(20.dp),
            )
        }
    }
}
