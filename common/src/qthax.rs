use cpp::{cpp, cpp_class};
//use qmetaobject::*;

cpp! {{
    #include <QtQml/QQmlEngine>
    #include <QtCore/QMetaType>
    #include "qthax.h"
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

cpp_class!(
    pub unsafe struct QGridLayout as "QGridLayout"
);
