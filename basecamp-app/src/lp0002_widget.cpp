#include "lp0002_widget.h"

#include <QQuickWidget>
#include <QVBoxLayout>
#include <QUrl>

Lp0002Widget::Lp0002Widget(QWidget* parent)
    : QWidget(parent)
    , quickWidget_(new QQuickWidget(this))
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->addWidget(quickWidget_);

    quickWidget_->setResizeMode(QQuickWidget::SizeRootObjectToView);
    quickWidget_->setSource(QUrl(QStringLiteral("qrc:/lp0002/qml/Lp0002PrivateMultisig.qml")));
}
