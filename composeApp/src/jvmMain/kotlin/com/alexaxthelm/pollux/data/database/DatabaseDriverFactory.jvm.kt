package com.alexaxthelm.pollux.data.database

import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import java.io.File

actual class DatabaseDriverFactory {
    actual fun createDriver(): SqlDriver {
        val dbDir = File(System.getProperty("user.home"), ".pollux")
        dbDir.mkdirs()
        val dbPath = File(dbDir, "pollux.db").absolutePath
        val driver = JdbcSqliteDriver("jdbc:sqlite:$dbPath")
        PolluxDatabase.Schema.create(driver)
        driver.execute(null, "PRAGMA foreign_keys = ON", 0)
        return driver
    }
}
