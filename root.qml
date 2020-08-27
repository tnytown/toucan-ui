/*
  import QtQuick 2.6
  import QtQuick.Window 2.0
  import QtQuick.Controls 1.4
  import QtQuick.Layouts 1.15
*/
import QtWidgets 1.0
import QtQml.Models 2.15
import FieldListModel 1.0

MainWindow {
	visible: true
	windowTitle: qsTr("toucan-ui")
	
	Widget {
		
		HBoxLayout {
			ListView {
				id: packetList
				model: listModel
				onClicked: {
					let x = listModel.on_clicked(index);
				}
			}
			TreeView {
				id: treeView
				model: detailModel
			}
		}
	}
}
