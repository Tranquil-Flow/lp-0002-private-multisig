#pragma once

#include <QWidget>

class QQuickWidget;

class Lp0002Widget : public QWidget
{
    Q_OBJECT

public:
    explicit Lp0002Widget(QWidget* parent = nullptr);
    ~Lp0002Widget() override = default;

private:
    QQuickWidget* quickWidget_;
};
