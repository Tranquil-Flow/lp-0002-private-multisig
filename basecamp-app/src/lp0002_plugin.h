#pragma once

#include "IComponent.h"
#include <QObject>
#include <QPointer>

class Lp0002Widget;

class Lp0002PrivateMultisigPlugin : public QObject, public IComponent
{
    Q_OBJECT
    Q_INTERFACES(IComponent)
    Q_PLUGIN_METADATA(IID IComponent_iid FILE "../metadata.json")

public:
    explicit Lp0002PrivateMultisigPlugin(QObject* parent = nullptr);
    ~Lp0002PrivateMultisigPlugin() override;

    Q_INVOKABLE QWidget* createWidget(LogosAPI* logosAPI = nullptr) override;
    void destroyWidget(QWidget* widget) override;

private:
    QPointer<Lp0002Widget> widget_;
};
