package com.alexaxthelm.pollux

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform