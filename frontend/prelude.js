//import QtQml 2.15

function delay(interval) {
	return new Promise((ok, err) => {
		try {
			let t = Qt.createQmlObject("import QtQml 2.15; Timer {}", app, "prelude.js");
			t.interval = interval; t.repeat = false;
			t.triggered.connect(ok);
			
			t.start();
		} catch(e) {
			err(e);
		}
	});
}

function createEditor(parent, labels) {
	let fields = [];
	for(let i = 0; i < labels.length; i++) {
		fields.push(`LineEdit { FormLayout.label: qsTr("${labels[i]}") }`);
	}

	let scpt = `
import QQmlDataWidgetMapper 1.0;
import QtWidgets 1.0;

FormLayout {
	GridLayout.row: 0
    GridLayout.column: 2

	id: editor
	property QQmlDataWidgetMapper imTheMap: QQmlDataWidgetMapper{}
    ${fields.join("\n")}
}
`;
 
	let editor = Qt.createQmlObject(scpt, parent, "prelude.js");
	let mapper = editor.imTheMap;
	
	return {editor, mapper};
}

function indexIsPacket(idx) {
	return idx.valid && idx.parent.valid && !idx.parent.parent.valid;
}

function indexIsPacketSet(idx) {
	return idx.valid && !idx.parent.valid;
}
