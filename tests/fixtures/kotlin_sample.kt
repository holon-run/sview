package com.example.app

import android.app.Activity
import kotlinx.coroutines.*

annotation class Screen

interface Presenter {
    fun start()
}

sealed class UiState

object AppConfig {
    const val name = "demo"
}

class MainActivity private constructor(
    private val presenter: Presenter
) : Activity() {
    var title: String = "Home"

    constructor() : this(DefaultPresenter())

    override fun onCreate() {
        presenter.start()
    }
}
