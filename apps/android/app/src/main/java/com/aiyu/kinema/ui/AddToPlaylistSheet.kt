package com.aiyu.kinema.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.PlaylistAdd
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.PlaylistPlay
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
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
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.R
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddToPlaylistSheet(
    itemId: Long,
    onDismiss: () -> Unit,
    onSuccess: (playlistName: String) -> Unit,
) {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    val playlistsLoader = rememberLoad { it.playlists() }
    val colors = MaterialTheme.colorScheme

    var isCreating by remember { mutableStateOf(false) }
    var newName by remember { mutableStateOf("") }
    var saving by remember { mutableStateOf(false) }

    fun addExisting(id: Long, name: String) {
        if (saving) return
        saving = true
        scope.launch {
            repository.request { it.addToPlaylist(id, listOf(itemId)) }
                .onSuccess {
                    repository.notifyChanged()
                    onSuccess(name)
                    onDismiss()
                }
            saving = false
        }
    }

    fun createAndAdd() {
        val trimmed = newName.trim()
        if (trimmed.isEmpty() || saving) return
        saving = true
        scope.launch {
            repository.request { it.createPlaylist(trimmed, listOf(itemId)) }
                .onSuccess {
                    repository.notifyChanged()
                    onSuccess(trimmed)
                    onDismiss()
                }
            saving = false
        }
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = colors.surfaceContainerLow,
    ) {
        Column(
            Modifier
                .fillMaxWidth()
                .padding(bottom = 24.dp),
        ) {
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = stringResource(R.string.playlist_add),
                    style = MaterialTheme.typography.titleLarge,
                )
            }

            if (isCreating) {
                Column(
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 24.dp, vertical = 8.dp),
                ) {
                    OutlinedTextField(
                        value = newName,
                        onValueChange = { newName = it },
                        label = { Text(stringResource(R.string.playlist_name)) },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                        keyboardActions = KeyboardActions(onDone = { createAndAdd() }),
                        modifier = Modifier.fillMaxWidth(),
                        shape = MaterialTheme.shapes.medium,
                    )
                    Spacer(Modifier.height(12.dp))
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        Button(
                            onClick = { isCreating = false; newName = "" },
                            modifier = Modifier.padding(end = 8.dp),
                        ) {
                            Text(stringResource(R.string.action_cancel))
                        }
                        Button(
                            onClick = ::createAndAdd,
                            enabled = newName.isNotBlank() && !saving,
                        ) {
                            Text(stringResource(R.string.action_create))
                        }
                    }
                }
            } else {
                ListItem(
                    headlineContent = { Text(stringResource(R.string.playlist_create), style = MaterialTheme.typography.titleMedium) },
                    leadingContent = {
                        Icon(
                            Icons.Rounded.Add,
                            null,
                            tint = colors.primary,
                            modifier = Modifier.size(24.dp),
                        )
                    },
                    colors = ListItemDefaults.colors(containerColor = Color.Transparent),
                    modifier = Modifier
                        .padding(horizontal = 12.dp)
                        .clip(MaterialTheme.shapes.large)
                        .clickable { isCreating = true },
                )
            }

            Loaded(playlistsLoader) { playlists ->
                LazyColumn(
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    items(playlists, key = { it.id }) { playlist ->
                        ListItem(
                            headlineContent = { Text(playlist.name, style = MaterialTheme.typography.titleMedium) },
                            supportingContent = {
                                Text(pluralStringResource(R.plurals.playlist_items_count, playlist.itemCount, playlist.itemCount))
                            },
                            leadingContent = {
                                Icon(Icons.Rounded.PlaylistPlay, null, modifier = Modifier.size(24.dp))
                            },
                            colors = ListItemDefaults.colors(containerColor = Color.Transparent),
                            modifier = Modifier
                                .clip(MaterialTheme.shapes.large)
                                .clickable { addExisting(playlist.id, playlist.name) },
                        )
                    }
                }
            }
        }
    }
}
