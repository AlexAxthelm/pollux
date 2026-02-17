package com.alexaxthelm.pollux.data.repository

import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import com.alexaxthelm.pollux.data.database.PolluxDatabase

fun createTestDatabase(): PolluxDatabase {
    val driver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
    PolluxDatabase.Schema.create(driver)
    driver.execute(null, "PRAGMA foreign_keys = ON", 0)
    return PolluxDatabase(driver)
}
