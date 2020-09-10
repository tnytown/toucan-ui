import QtQml.Models 2.15
import QtWidgets 1.0
import QtQml 2.15
import QtQuick 2.15 as Quick
import QQmlDataWidgetMapper 1.0

import "./prelude.js" as Z
import me.unown.qthax 1.0

MainWindow {
	visible: true
	windowTitle: qsTr("toucan-ui")

	function debug_open_directory() {
		const DEBUG_DIR = "/Users/apan/dev/svp/toucan/toucan-ui/data";
		
	    for(; !packetList.model; packetList.model = app.open_directory(DEBUG_DIR));

		// TODO(aptny): expose column hiding as macro attr
		for(let x of [1, 2]) {
			packetList.hideColumn(x);
		}
		
		statusBar.showMessage("Ready.");
	}
	
	Component.onCompleted: Z.delay(0).then(debug_open_directory)

	MenuBar {
		Menu {
			title: qsTr("&File")

			Action {
				text: qsTr("&Open")
				onTriggered: {
					let file = FileDialog.getExistingDirectory()
					packetList.model = app.open_directory(file)
				}
			}
		}
	}
	
	Widget {
		GridLayout {
			id: rootLayout
			
			Component.onCompleted: {
				let cols = {0: 0, 1: 4, 2: 1};
				for(const [c, f] of Object.entries(cols)) {
					QtHax.gridColumnStretch(this, c, f);
				}
			}

			// packet list view
			TreeView {
				GridLayout.row: 0
				GridLayout.column: 0
				
				id: packetList
				
				onClicked: {
					let mdl = packetList.model;
					let labels = [];
					for(let i = 0; ; i++) {
						const ROLE_FIELD_LABEL = 0x0102;
						let col = mdl.index(index.row, i, index.parent);
						let data = mdl.data(col, ROLE_FIELD_LABEL);
						if(!data) break;
						labels.push(data);
					}
					
					let {editor, mapper} = Z.createEditor(rootLayout, labels);
					QtHax.gridReplaceLayout(rootLayout, editor, 0, 2);

					mapper.setModel(mdl);
					for(let i = 0; i < labels.length; i++) {
						mapper.addMapping(editor.data[i], i);
					}
					mapper.setRootIndex(index.parent);
					mapper.setCurrentIndex(index.row);

					// only set the packet view if the data is a packet.
					if(!Z.indexIsPacket(index))
						return;
					
					packetDetail.model = mdl;
					packetDetail.setRootIndex(index);
				}

				Component.onCompleted: {
					headerHidden = true;
				}
			}

			// packet detail view
			TreeView {
				GridLayout.row: 0
				GridLayout.column: 1
				
				id: packetDetail
			}

			// editor
		}
	}
	StatusBar {
		
		id: statusBar
	}
}
