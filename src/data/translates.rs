use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    English,
    Russian,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translator {
    pub language: Language,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn add_server(&self) -> String {
        match self.language {
            Language::English => "Add server".to_string(),
            Language::Russian => "Добавить сервер".to_string(),
        }
    }

    pub fn name(&self) -> String {
        match self.language {
            Language::English => "Name".to_string(),
            Language::Russian => "Имя".to_string(),
        }
    }

    pub fn server_address(&self) -> String {
        match self.language {
            Language::English => "Server address".to_string(),
            Language::Russian => "Адрес сервера".to_string(),
        }
    }

    pub fn port(&self) -> String {
        match self.language {
            Language::English => "Port".to_string(),
            Language::Russian => "Порт".to_string(),
        }
    }

    pub fn user(&self) -> String {
        match self.language {
            Language::English => "User".to_string(),
            Language::Russian => "Пользователь".to_string(),
        }
    }

    pub fn password(&self) -> String {
        match self.language {
            Language::English => "Password".to_string(),
            Language::Russian => "Пароль".to_string(),
        }
    }

    pub fn service_database(&self) -> String {
        match self.language {
            Language::English => "Service database".to_string(),
            Language::Russian => "Сервисная база данных".to_string(),
        }
    }

    pub fn language(&self) -> String {
        match self.language {
            Language::English => "Language".to_string(),
            Language::Russian => "Язык".to_string(),
        }
    }

    pub fn settings(&self) -> String {
        match self.language {
            Language::English => "Settings".to_string(),
            Language::Russian => "Настройки".to_string(),
        }
    }

    pub fn scale_factor(&self) -> String {
        match self.language {
            Language::English => "Scale factor".to_string(),
            Language::Russian => "Масштаб".to_string(),
        }
    }

    pub fn theme(&self) -> String {
        match self.language {
            Language::English => "Theme".to_string(),
            Language::Russian => "Тема".to_string(),
        }
    }

    pub fn light(&self) -> String {
        match self.language {
            Language::English => "Light".to_string(),
            Language::Russian => "Светлая".to_string(),
        }
    }

    pub fn dark(&self) -> String {
        match self.language {
            Language::English => "Dark".to_string(),
            Language::Russian => "Темная".to_string(),
        }
    }

    pub fn name_is_required(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Name is required".to_string(),
            Language::Russian => "Имя обязательно".to_string(),
        })
    }

    pub fn name_must_be_less_than_32_characters(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Name must be less than 32 characters".to_string(),
            Language::Russian => "Имя должно быть меньше 32 символов".to_string(),
        })
    }

    pub fn name_must_be_unique(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Name must be unique".to_string(),
            Language::Russian => "Имя должно быть уникальным".to_string(),
        })
    }

    pub fn port_is_required(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Port is required".to_string(),
            Language::Russian => "Порт обязателен".to_string(),
        })
    }

    pub fn user_is_required(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "User is required".to_string(),
            Language::Russian => "Пользователь обязателен".to_string(),
        })
    }

    pub fn ip_is_required(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "IP is required".to_string(),
            Language::Russian => "IP обязателен".to_string(),
        })
    }

    pub fn incorrect_port_value(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Incorrect port value".to_string(),
            Language::Russian => "Некорректный порт".to_string(),
        })
    }

    pub fn service_database_is_required(&self) -> String {
        format!("- {}", match self.language {
            Language::English => "Service database is required".to_string(),
            Language::Russian => "Сервисная база данных обязательна".to_string(),
        })
    }

    pub fn save(&self) -> String {
        match self.language {
            Language::English => "Save".to_string(),
            Language::Russian => "Сохранить".to_string(),
        }
    }

    pub fn open(&self) -> String {
        match self.language {
            Language::English => "Open".to_string(),
            Language::Russian => "Открыть".to_string(),
        }
    }

    pub fn back(&self) -> String {
        match self.language {
            Language::English => "Back".to_string(),
            Language::Russian => "Назад".to_string(),
        }
    }

    pub fn delete_server(&self) -> String {
        match self.language {
            Language::English => "Delete server".to_string(),
            Language::Russian => "Удалить сервер".to_string(),
        }
    }

    pub fn edit_server(&self) -> String {
        match self.language {
            Language::English => "Edit server".to_string(),
            Language::Russian => "Редактировать сервер".to_string(),
        }
    }

    pub fn delete_server_confirmation(&self) -> String {
        match self.language {
            Language::English => "Are you sure you want to delete this server?".to_string(),
            Language::Russian => "Вы уверены, что хотите удалить этот сервер?".to_string(),
        }
    }

    pub fn yes(&self) -> String {
        match self.language {
            Language::English => "Yes".to_string(),
            Language::Russian => "Да".to_string(),
        }
    }

    pub fn no(&self) -> String {
        match self.language {
            Language::English => "No".to_string(),
            Language::Russian => "Нет".to_string(),
        }
    }

    pub fn text_viewer(&self) -> String {
        match self.language {
            Language::English => "Text viewer".to_string(),
            Language::Russian => "Текстовый просмотрщик".to_string(),
        }
    }

    pub fn copy(&self) -> String {
        match self.language {
            Language::English => "Copy".to_string(),
            Language::Russian => "Копировать".to_string(),
        }
    }

    pub fn close(&self) -> String {
        match self.language {
            Language::English => "Close".to_string(),
            Language::Russian => "Закрыть".to_string(),
        }
    }

    pub fn welcome(&self) -> String {
        match self.language {
            Language::English => "Welcome to Rs-Postgres: Rust-based PostgreSQL client.".to_string(),
            Language::Russian => "Добро пожаловать в Rs-Postgres: Rust-based PostgreSQL клиент.".to_string(),
        }
    }

    pub fn features(&self) -> String {
        match self.language {
            Language::English => "Features".to_string(),
            Language::Russian => "Функции".to_string(),
        }
    }

    pub fn features_content(&self) -> String {
        match self.language {
            Language::English => r#"• Lightweight and fast
• Secure encryption of server credentials
• Connect to multiple PostgreSQL servers
• Manage databases through GUI
• Execute SQL queries with results view"#.to_string(),
            Language::Russian => r#"• Легкий и быстрый
• Безопасное шифрование учетных данных сервера
• Подключение ко множеству серверов PostgreSQL
• Управление базами данных через графический интерфейс
• Выполнение SQL-запросов с просмотром результатов"#.to_string(),
        }
    }

    pub fn get_started(&self) -> String {
        match self.language {
            Language::English => "Getting started".to_string(),
            Language::Russian => "Начало работы".to_string(),
        }
    }

    pub fn get_started_content(&self) -> String {
        match self.language {
            Language::English => r#"1. Click "Add server" in left panel
2. Enter server connection parameters
3. Select database in connection tree
4. Start working with SQL queries by clicking "SQL Query" button or choosing preset script"#.to_string(),
            Language::Russian => r#"1. Нажмите "Добавить сервер" на левой панели
2. Введите параметры подключения к серверу
3. Выберите базу данных в дереве соединений.
4. Начните работу с SQL-запросами, нажав кнопку "SQL Query" или выбрав предустановленный скрипт."#.to_string(),
        }
    }

    pub fn resources(&self) -> String {
        match self.language {
            Language::English => "Resources".to_string(),
            Language::Russian => "Ресурсы".to_string(),
        }
    }

    pub fn github(&self) -> String {
        match self.language {
            Language::English => "🐙 GitHub".to_string(),
            Language::Russian => "🐙 GitHub".to_string(),
        }
    }

    pub fn open_repo(&self) -> String {
        match self.language {
            Language::English => "Open repository".to_string(),
            Language::Russian => "Открыть репозиторий".to_string(),
        }
    }

    pub fn license(&self) -> String {
        match self.language {
            Language::English => "📝 License".to_string(),
            Language::Russian => "📝 Лицензия".to_string(),
        }
    }

    pub fn open_license(&self) -> String {
        match self.language {
            Language::English => "Open license".to_string(),
            Language::Russian => "Открыть лицензию".to_string(),
        }
    }

    pub fn support(&self) -> String {
        match self.language {
            Language::English => "📨 Support".to_string(),
            Language::Russian => "📨 Поддержка".to_string(),
        }
    }

    pub fn open_support(&self) -> String {
        match self.language {
            Language::English => "Open support".to_string(),
            Language::Russian => "Открыть поддержку".to_string(),
        }
    }

    pub fn version(&self, version: impl ToString) -> String {
        match self.language {
            Language::English => format!("Version {}", version.to_string()),
            Language::Russian => format!("Версия {}", version.to_string()),
        }
    }

    pub fn run_f5(&self) -> String {
        match self.language {
            Language::English => "Run (F5)".to_string(),
            Language::Russian => "Выполнить (F5)".to_string(),
        }
    }

    pub fn clear(&self) -> String {
        match self.language {
            Language::English => "Clear".to_string(),
            Language::Russian => "Очистить".to_string(),
        }
    }

    pub fn file(&self) -> String {
        match self.language {
            Language::English => "File:".to_string(),
            Language::Russian => "Файл:".to_string(),
        }
    }

    pub fn running(&self) -> String {
        match self.language {
            Language::English => "Running...".to_string(),
            Language::Russian => "Выполняется...".to_string(),
        }
    }

    pub fn success(&self) -> String {
        match self.language {
            Language::English => "Success".to_string(),
            Language::Russian => "Успешно".to_string(),
        }
    }

    pub fn time(&self, time: impl ToString) -> String {
        match self.language {
            Language::English => format!("Time: {} ms", time.to_string()),
            Language::Russian => format!("Время: {} ms", time.to_string()),
        }
    }

    pub fn rows(&self, rows: impl ToString) -> String {
        match self.language {
            Language::English => format!("Rows: {}", rows.to_string()),
            Language::Russian => format!("Строки: {}", rows.to_string()),
        }
    }

    pub fn click_to_copy(&self) -> String {
        match self.language {
            Language::English => "Click to copy".to_string(),
            Language::Russian => "Нажмите, чтобы скопировать".to_string(),
        }
    }

    pub fn no_data_returned(&self) -> String {
        match self.language {
            Language::English => "No data returned".to_string(),
            Language::Russian => "Нет данных".to_string(),
        }
    }

    pub fn error(&self) -> String {
        match self.language {
            Language::English => "Error!".to_string(),
            Language::Russian => "Ошибка!".to_string(),
        }
    }

    pub fn clear_storage(&self) -> String {
        match self.language {
            Language::English => "Clear storage".to_string(),
            Language::Russian => "Очистить хранилище".to_string(),
        }
    }

    pub fn clear_storage_confirmation(&self) -> String {
        match self.language {
            Language::English => "Do you want to clear storage? This action is irreversible.".to_string(),
            Language::Russian => "Вы уверены, что хотите очистить хранилище? Это действие необратимо.".to_string(),
        }
    }

    pub fn login(&self) -> String {
        match self.language {
            Language::English => "Login".to_string(),
            Language::Russian => "Вход".to_string(),
        }
    }

    pub fn enter_encryption_password(&self) -> String {
        match self.language {
            Language::English => "Enter encryption password:".to_string(),
            Language::Russian => "Введите пароль шифрования:".to_string(),
        }
    }

    pub fn create_encryption_password(&self) -> String {
        match self.language {
            Language::English => "Create encryption password:".to_string(),
            Language::Russian => "Задайте пароль шифрования:".to_string(),
        }
    }

    pub fn incorrect_password_hash_mismatch(&self) -> String {
        match self.language {
            Language::English => "Incorrect password: hash mismatch".to_string(),
            Language::Russian => "Неверный пароль: несоответствие хэша".to_string(),
        }
    }

    pub fn servers(&self) -> String {
        match self.language {
            Language::English => "Servers".to_string(),
            Language::Russian => "Серверы".to_string(),
        }
    }

    pub fn databases(&self) -> String {
        match self.language {
            Language::English => "Databases".to_string(),
            Language::Russian => "Базы данных".to_string(),
        }
    }

    pub fn tables(&self) -> String {
        match self.language {
            Language::English => "Tables".to_string(),
            Language::Russian => "Таблицы".to_string(),
        }
    }

    pub fn scripts(&self) -> String {
        match self.language {
            Language::English => "Scripts".to_string(),
            Language::Russian => "Скрипты".to_string(),
        }
    }

    pub fn get_columns(&self) -> String {
        match self.language {
            Language::English => "Get columns".to_string(),
            Language::Russian => "Получить столбцы".to_string(),
        }
    }

    pub fn delete(&self) -> String {
        match self.language {
            Language::English => "Delete".to_string(),
            Language::Russian => "Удалить".to_string(),
        }
    }

    pub fn edit(&self) -> String {
        match self.language {
            Language::English => "Edit".to_string(),
            Language::Russian => "Редактировать".to_string(),
        }
    }

    pub fn reload(&self) -> String {
        match self.language {
            Language::English => "Reload".to_string(),
            Language::Russian => "Перезагрузить".to_string(),
        }
    }

    pub fn change_password(&self) -> String {
        match self.language {
            Language::English => "Change password".to_string(),
            Language::Russian => "Сменить пароль".to_string(),
        }
    }

    pub fn old_password(&self) -> String {
        match self.language {
            Language::English => "Old password".to_string(),
            Language::Russian => "Старый пароль".to_string(),
        }
    }

    pub fn new_password(&self) -> String {
        match self.language {
            Language::English => "New password".to_string(),
            Language::Russian => "Новый пароль".to_string(),
        }
    }

    pub fn confirm_password(&self) -> String {
        match self.language {
            Language::English => "Confirm password".to_string(),
            Language::Russian => "Подтвердите пароль".to_string(),
        }
    }

    pub fn passwords_do_not_match(&self) -> String {
        match self.language {
            Language::English => "Passwords do not match".to_string(),
            Language::Russian => "Пароли не совпадают".to_string(),
        }
    }

}
