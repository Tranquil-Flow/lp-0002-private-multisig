#include "lp0002_widget.h"

#include "lp0002_backend.h"

#include <QQmlContext>
#include <QQuickWidget>
#include <QVBoxLayout>
#include <QUrl>

Lp0002Widget::Lp0002Widget(QWidget* parent)
    : QWidget(parent)
    , quickWidget_(new QQuickWidget(this))
    , backend_(new Lp0002Backend(this))
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->addWidget(quickWidget_);

    quickWidget_->setResizeMode(QQuickWidget::SizeRootObjectToView);
    quickWidget_->rootContext()->setContextProperty(QStringLiteral("lp0002Backend"), backend_);
    quickWidget_->setSource(QUrl(QStringLiteral("qrc:/lp0002/qml/Lp0002PrivateMultisig.qml")));
}
