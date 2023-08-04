# Описание
Пример внешней компоненты для **1С:Предприятие 8** по технологии **Native API** на языке **Rust**, изначально написанный   [пользователем **medigor**](https://github.com/medigor/example-native-api-rs), мною форкнут, т.к. мне не понравилась структура проекта и не доставало имплементации объекта соединения с базой (для отправления внешних и др.).

Стараюсь всё реализовывать идиоматически, насколько хватает времени, желания и знаний, буду рад корректировкам :)

## Размер .dll
Релизная сборка с оптимизациями на размер 
| Использование библиотеки `ureq` | Без сжатия | Сжатие с помощью [UPX](https://upx.github.io/) |
| ------------------------------- | ---------- | ---------------------------------------------- |
| Нет                             | 243200 B   | 115200 B                                       |
| Да                              | 1379328 B  | 738816 B                                       |


## Поддержка платформ
- Windows x64 - msvc работает, gnu не тестировал
- Windows x32 - msvc работает, gnu не тестировал
- Linux x64 - не тестировал.
- Linux x32 - не тестировал.
- MacOS - не тестировал.


## Пример части кода, описывающего компоненту

```rust
impl MyAddInDescription {
    pub fn new() -> Self {
        Self {
            name: &utf16_null!("MyAddIn"),          // имя класса в 1С
            connection: Arc::new(None),
            functions: Self::generate_func_list(),
            props: Self::generate_prop_list(),

            some_prop_container: 0,
        }
    }
}

impl MyAddInDescription {
    pub fn generate_func_list() -> Vec<FunctionListElement> {
        vec![
            FunctionListElement {                           
                description: ComponentFuncDescription::new::<0>( // количество аргументов
                    &["Итерировать", "Iterate"],    // название свойства на русском и английском языках
                    false,                          // возвращает значения
                    &[],                            // значения по умолчанию, Option
                ),
                callback: Self::iterate,            // функция обработчик
            },
        ]
    }

    pub fn generate_prop_list() -> Vec<PropListElement> {
        vec![PropListElement {
            description: ComponentPropDescription {
                names: &["Property", "Свойство"],   // название свойства на русском и английском языках
                readable: true,                     // 1С может читать значение
                writable: true,                     // 1С может записывать значение
            },
            get_callback: Some(Self::get_prop),     // функция обработчик чтения значения
            set_callback: Some(Self::set_prop),     // функция обработчик записи значения
        }]
    }

    fn iterate(
        &mut self,
        _params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        if self.some_prop_container >= 105 {
            return Err(eyre!("Prop is too big"));
        }
        self.some_prop_container += 1;
        log::info!("Prop is now {}", self.some_prop_container);
        Ok(None)
    }

    fn get_prop(&self) -> Option<ParamValue> {
        Some(ParamValue::I32(self.some_prop_container))
    }

    fn set_prop(&mut self, value: &ParamValue) -> bool {
        match value {
            ParamValue::I32(val) => {
                self.some_prop_container = *val;
                true
            }
            _ => false,
        }
    }
}
```





#### Другие ресурсы
* [Документция на ИТС](https://its.1c.ru/db/metod8dev#content:3221:hdoc)
* [Шаблон компоненты на C++ от Infactum](https://github.com/Infactum/addin-template)

>## Далее сказанное [изначальным автором](https://github.com/medigor/example-native-api-rs)
>## Преимущества по сравнению с компонентой на C++
>* Преимущества самого языка *Rust* и его экосистемы (более современный и безопасный язык, удобный пакетный менеджер)
>* Для Windows не требуется msvc (напомню, что организации должны иметь лицензию)
>* Собирается полностью с использованием свободных инструментов
>* На linux можно собирать для windows, соответственно удобно использовать в CI контейнеры linux
>
>## Обзор
>Компоненты по технологии *Native API* предполагают разработку на языке *C++*, т.к. компонента должна принимать и возвращать указатели на виртуальные классы *C++*. Компонента для windows должна собираться только компилятором msvc, а для linux и macos подойдет gcc/clang.
>Как известно, взаимодействие *Rust* с *C++* из коробки не поддерживается. 
>
>Одним из вариантов было использовать [cxx](https://github.com/dtolnay/cxx) или подобные библиотеки. Это также бы потребовало использовать msvc.
>
>Другой вариант - вручную реализовать виртуальные таблицы, именно этот вариант и реализован.
На [godbolt](https://godbolt.org/z/KM3jaWMWs) можно посмотреть, как выглядят виртуальные таблицы для разных компиляторов. Виртуальные таблицы *msvc* отличаются от *gcc*/*clang*, при этом *gcc* и *clang* используют одинаковое ABI. Виртуальные таблицы реализованы в объеме достаточном для создания компоненты.
