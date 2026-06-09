import QtQuick 2.15
import QtQuick.Controls.Basic 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: root
    color: "#0d1117"
    implicitWidth: 1040
    implicitHeight: 720

    property var latest: ({})
    property var execution: ({ status: "NotExecuted", message: "Generate a threshold journal first." })
    property var replay: ({ status: "Unchecked" })

    function approvals() {
        return [aliceBox.checked, borisBox.checked, cyraBox.checked, daraBox.checked, evanBox.checked]
    }

    function prove() {
        latest = lp0002Backend.proveThreshold(thresholdBox.value, approvals(), proposalField.text, actionField.text)
        execution = ({ status: "Ready", message: "Journal generated. Execute once to mark replay state." })
        replay = lp0002Backend.checkReplay(latest.journal)
    }

    function executeOnce() {
        if (!latest.journal) {
            prove()
        }
        execution = lp0002Backend.executeJournal(latest.journal)
        replay = lp0002Backend.checkReplay(latest.journal)
    }

    function resetReplay() {
        execution = lp0002Backend.resetReplay()
        if (latest.journal) {
            replay = lp0002Backend.checkReplay(latest.journal)
        }
    }

    Component.onCompleted: prove()

    ScrollView {
        anchors.fill: parent

        ColumnLayout {
            width: Math.max(root.width - 56, 900)
            spacing: 14
            anchors.margins: 28

            Label {
                text: "LP-0002 Private M-of-N Multisig"
                color: "#f7fbff"
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }

            Label {
                text: "Run the threshold gate directly in Basecamp: local approvers create unlinkable nullifiers, the public journal exposes only counts and hashes, and the wrapper executes once."
                color: "#b9c6d8"
                wrapMode: Text.WordWrap
                font.pixelSize: 15
                Layout.fillWidth: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 12

                Rectangle {
                    Layout.preferredWidth: 320
                    Layout.fillHeight: true
                    radius: 8
                    color: "#151b24"
                    border.color: "#2c3544"

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 12

                        Label {
                            text: "Proposal"
                            color: "#eef5ff"
                            font.pixelSize: 18
                            font.bold: true
                        }

                        Label { text: "Threshold"; color: "#9fb0c6" }
                        SpinBox {
                            id: thresholdBox
                            from: 1
                            to: 5
                            value: 2
                            editable: true
                            palette.text: "#f7fbff"
                            palette.buttonText: "#f7fbff"
                            palette.base: "#0b0f16"
                            palette.button: "#1f2937"
                            onValueModified: prove()
                        }

                        Label { text: "Proposal ID"; color: "#9fb0c6" }
                        TextField {
                            id: proposalField
                            Layout.fillWidth: true
                            text: "grant-42"
                            color: "#f7fbff"
                            onTextEdited: prove()
                            background: Rectangle { color: "#0b0f16"; radius: 6; border.color: "#334052" }
                        }

                        Label { text: "Action"; color: "#9fb0c6" }
                        TextArea {
                            id: actionField
                            Layout.fillWidth: true
                            Layout.preferredHeight: 86
                            text: "transfer 42 LOGOS to shielded treasury recipient"
                            wrapMode: Text.WordWrap
                            color: "#f7fbff"
                            onTextChanged: prove()
                            background: Rectangle { color: "#0b0f16"; radius: 6; border.color: "#334052" }
                        }

                        Label {
                            text: "Local approvers"
                            color: "#eef5ff"
                            font.pixelSize: 16
                            font.bold: true
                        }

                        CheckBox { id: aliceBox; text: "Alice"; checked: true; palette.text: "#f7fbff"; onToggled: prove() }
                        CheckBox { id: borisBox; text: "Boris"; checked: false; palette.text: "#f7fbff"; onToggled: prove() }
                        CheckBox { id: cyraBox; text: "Cyra"; checked: true; palette.text: "#f7fbff"; onToggled: prove() }
                        CheckBox { id: daraBox; text: "Dara"; checked: false; palette.text: "#f7fbff"; onToggled: prove() }
                        CheckBox { id: evanBox; text: "Evan"; checked: false; palette.text: "#f7fbff"; onToggled: prove() }

                        RowLayout {
                            Layout.fillWidth: true
                            Button { text: "Prove"; Layout.fillWidth: true; palette.buttonText: "#111827"; onClicked: prove() }
                            Button { text: "Execute"; Layout.fillWidth: true; palette.buttonText: "#111827"; onClicked: executeOnce() }
                        }

                        Button {
                            text: "Reset replay state"
                            Layout.fillWidth: true
                            palette.buttonText: "#111827"
                            onClicked: resetReplay()
                        }
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 88
                            radius: 8
                            color: latest.ok ? "#10291e" : "#2a1b1d"
                            border.color: latest.ok ? "#39d98a" : "#ff6b6b"

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 14
                                Label { text: "Proof gate"; color: "#9fb0c6"; font.pixelSize: 13 }
                                Label { text: latest.status || "Unknown"; color: "#f7fbff"; font.pixelSize: 21; font.bold: true }
                                Label {
                                    text: "Receipt " + (latest.receipt_id || "pending")
                                    color: "#c7d4e6"
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 88
                            radius: 8
                            color: execution.ok ? "#10291e" : "#1a2230"
                            border.color: execution.ok ? "#39d98a" : "#40516b"

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 14
                                Label { text: "Execution"; color: "#9fb0c6"; font.pixelSize: 13 }
                                Label { text: execution.status || "NotExecuted"; color: "#f7fbff"; font.pixelSize: 21; font.bold: true }
                                Label {
                                    text: execution.message || replay.status || ""
                                    color: "#c7d4e6"
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 178
                        radius: 8
                        color: "#141a23"
                        border.color: "#2d3849"

                        GridLayout {
                            anchors.fill: parent
                            anchors.margins: 16
                            columns: 2
                            columnSpacing: 18
                            rowSpacing: 10

                            Label { text: "Approval count"; color: "#9fb0c6" }
                            Label { text: latest.journal ? latest.journal.approval_count + " / " + latest.journal.threshold : "-"; color: "#f7fbff"; font.bold: true }

                            Label { text: "Replay check"; color: "#9fb0c6" }
                            Label { text: replay.status || "Unchecked"; color: replay.ok ? "#39d98a" : "#ffbf69"; font.bold: true }

                            Label { text: "Member root"; color: "#9fb0c6" }
                            Label { text: latest.journal ? latest.journal.member_root : "-"; color: "#d8e7ff"; elide: Text.ElideMiddle; Layout.fillWidth: true }

                            Label { text: "Action hash"; color: "#9fb0c6" }
                            Label { text: latest.journal ? latest.journal.action_hash : "-"; color: "#d8e7ff"; elide: Text.ElideMiddle; Layout.fillWidth: true }

                            Label { text: "Privacy boundary"; color: "#9fb0c6" }
                            Label { text: latest.privacy || "-"; color: "#d8e7ff"; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 330
                        radius: 8
                        color: "#10151d"
                        border.color: "#2d3849"

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 16
                            spacing: 10

                            Label {
                                text: "Public verifier journal"
                                color: "#eef5ff"
                                font.pixelSize: 18
                                font.bold: true
                            }

                            TextArea {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                readOnly: true
                                color: "#d9ffe2"
                                font.family: "monospace"
                                font.pixelSize: 13
                                text: latest.journal ? JSON.stringify(latest.journal, null, 2) : "{}"
                                background: Rectangle { color: "#070a10"; radius: 6; border.color: "#1f2a38" }
                            }
                        }
                    }
                }
            }
        }
    }
}
