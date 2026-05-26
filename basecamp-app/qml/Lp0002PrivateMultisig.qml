import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#10131a"
    implicitWidth: 960
    implicitHeight: 640

    property var steps: [
        "1. Create private multisig: publish member root + threshold",
        "2. Propose action: bind proposal id, action hash, context",
        "3. Approve privately: signer proves membership without identity leak",
        "4. Prove threshold: RISC0 receipt commits to nullifiers + journal",
        "5. Execute once: LEZ wrapper checks commitment and replay state"
    ]

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 28
        spacing: 18

        Label {
            text: "LP-0002 Private M-of-N Multisig"
            color: "#f5f7ff"
            font.pixelSize: 30
            font.bold: true
            Layout.fillWidth: true
        }

        Label {
            text: "Shielded approvals for Logos Execution Zone: approvers prove threshold membership while revealing only nullifiers and execution-safe public state."
            color: "#b9c0d4"
            wrapMode: Text.WordWrap
            font.pixelSize: 16
            Layout.fillWidth: true
        }

        Rectangle {
            color: "#171b25"
            border.color: "#30384b"
            radius: 12
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 12

                Repeater {
                    model: root.steps
                    delegate: Rectangle {
                        Layout.fillWidth: true
                        height: 48
                        radius: 8
                        color: index === 3 ? "#1d3142" : "#202637"
                        border.color: index === 3 ? "#5cc8ff" : "#394357"

                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            anchors.right: parent.right
                            anchors.rightMargin: 16
                            text: modelData
                            color: "#edf2ff"
                            font.pixelSize: 15
                            elide: Text.ElideRight
                        }
                    }
                }

                Item { Layout.fillHeight: true }

                Label {
                    text: "Run ./demo.sh for the clone-and-run consumer app, and scripts/demo-heavy-lane.sh for RISC0_DEV_MODE=0 proof artifact verification plus LEZ wrapper inclusion evidence."
                    color: "#91d7ff"
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }
    }
}
