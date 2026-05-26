#include "lp0002_plugin.h"
#include "lp0002_widget.h"

#include <QDebug>

Lp0002PrivateMultisigPlugin::Lp0002PrivateMultisigPlugin(QObject* parent)
    : QObject(parent)
{
    qDebug() << "LP-0002 private multisig plugin created";
}

Lp0002PrivateMultisigPlugin::~Lp0002PrivateMultisigPlugin()
{
    destroyWidget(widget_.data());
}

QWidget* Lp0002PrivateMultisigPlugin::createWidget(LogosAPI*)
{
    if (!widget_) {
        widget_ = new Lp0002Widget();
    }
    return widget_;
}

void Lp0002PrivateMultisigPlugin::destroyWidget(QWidget* widget)
{
    if (widget) {
        delete widget;
        if (widget == widget_) {
            widget_.clear();
        }
    }
}
