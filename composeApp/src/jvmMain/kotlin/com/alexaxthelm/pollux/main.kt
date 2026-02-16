package com.alexaxthelm.pollux

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application

fun main() = application {
    Window(
        onCloseRequest = ::exitApplication,
        title = "pollux",
    ) {
        App()
    }
}