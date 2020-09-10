#include <QtWidgets/QDataWidgetMapper>
#include <QtWidgets/QLayoutItem>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QWidget>
#include <QtCore/QModelIndex>
#include <QtCore/QAbstractItemModel>
#include <QtQml/QJSEngine>
#include <QtQml/QQmlEngine>

class QQmlDataWidgetMapper: public QDataWidgetMapper {
	Q_OBJECT

public:
	Q_INVOKABLE void addMapping(QWidget* widget, int section) {
		QDataWidgetMapper::addMapping(widget, section);
	}
    Q_INVOKABLE void setModel(QAbstractItemModel *model) {
		QDataWidgetMapper::setModel(model);
	}

	Q_INVOKABLE void setRootIndex(const QModelIndex &index) {
		QDataWidgetMapper::setRootIndex(index);
	}

	Q_INVOKABLE QModelIndex rootIndex() const {
		return QDataWidgetMapper::rootIndex();
	}
};

class QtHax: public QObject {
    Q_OBJECT

public:
    QtHax(QObject* parent = 0) : QObject(parent) {}

    ~QtHax() {}

	Q_INVOKABLE void gridColumnStretch(QGridLayout* layout, int col, int stretch) {
		layout->setColumnStretch(col, stretch);
	}

	Q_INVOKABLE void gridReplaceLayout(QGridLayout* grid, QLayout* layout, int r, int c) {
		QLayoutItem* current = grid->itemAtPosition(r, c);
		if(current) {
			grid->removeItem(current);
			QLayout* layoutObj = current->layout(); // shrug
			
			if(!layoutObj) return;

			QLayoutItem* child;
			while(!layoutObj->isEmpty() && (child = layoutObj->takeAt(0)) != nullptr) {
				deleteIfNonNull(child->widget());
				deleteIfNonNull(child->layout());
			}
			
			layoutObj->setParent(nullptr);
			layoutObj->deleteLater();
		}

		// reparent
		QLayout *parent = layout->parentWidget()->layout();
		parent->removeItem(layout);
		layout->setParent(nullptr);
		
		grid->addLayout(layout, r, c);
	}
private:
	void deleteIfNonNull(QObject *data) {
		if(data) data->deleteLater();
	}
};
