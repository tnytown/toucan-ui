use cpp::{cpp, cpp_class};
use qmetaobject::*;

cpp! {{
    #include <QtQml/QQmlEngine>
    #include <QtQml/QQmlApplicationEngine>
    #include <QtQuick/QtQuick>
    #include <QtCore/QMetaType>
    #include "qthax.h"

    struct QmlEngineHolder {
        std::unique_ptr<QApplication> app;
        std::unique_ptr<QQmlApplicationEngine> engine;
        std::unique_ptr<QQuickView> view;
    };
}}

/*cpp_class!(
    pub unsafe struct QDataWidgetMapper as "QDataWidgetMapper"
);*/

pub fn register_types() {
    cpp!(unsafe [] -> () as "void" {
        qRegisterMetaType<QGridLayout*>("const QGridLayout*");
        qmlRegisterSingletonType<QtHax>("me.unown.qthax", 1, 0, "QtHax", [](QQmlEngine *engine, QJSEngine *scriptEngine) -> QObject* {
            Q_UNUSED(engine)
            Q_UNUSED(scriptEngine)

            QtHax *hax = new QtHax();
            return hax;
        });
        qmlRegisterType<QQmlDataWidgetMapper>("QQmlDataWidgetMapper", 1, 0, "QQmlDataWidgetMapper");
    });
}

pub trait JSExposeExt {
    fn js_expose<T: QObject>(&mut self, obj: T) -> QJSValue;
    fn js_throw(&mut self, err: &str);

    fn js_map_or_throw<T, E>(&mut self, res: Result<T, E>) -> QJSValue
    where
        T: QObject,
        E: ToString;
}

impl JSExposeExt for Option<QmlEngine> {
    fn js_expose<T: QObject>(&mut self, obj: T) -> QJSValue {
        self.as_mut().map_or(false.into(), |e| e.new_qobject(obj))
    }

    fn js_throw(&mut self, err: &str) {
        self.as_mut().map(|h| {
            let err: QString = err.into();
            cpp!(unsafe [h as "QmlEngineHolder *", err as "QString"] {
                h->engine->throwError(err);
            });
        });
    }

    fn js_map_or_throw<T, E>(&mut self, res: Result<T, E>) -> QJSValue
    where
        T: QObject,
        E: ToString,
    {
        match res {
            Ok(t) => self.js_expose(t),
            Err(e) => {
                self.js_throw(&e.to_string());
                false.into()
            }
        }
    }
}

cpp_class!(
    pub unsafe struct QGridLayout as "QGridLayout"
);
