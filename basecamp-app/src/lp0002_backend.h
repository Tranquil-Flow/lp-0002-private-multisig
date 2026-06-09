#pragma once

#include <QObject>
#include <QSet>
#include <QVariantList>
#include <QVariantMap>

class Lp0002Backend : public QObject
{
    Q_OBJECT

public:
    explicit Lp0002Backend(QObject* parent = nullptr);

    Q_INVOKABLE QVariantMap proveThreshold(int threshold,
                                           const QVariantList& approvals,
                                           const QString& proposalId,
                                           const QString& actionText);
    Q_INVOKABLE QVariantMap executeJournal(const QVariantMap& journal);
    Q_INVOKABLE QVariantMap checkReplay(const QVariantMap& journal) const;
    Q_INVOKABLE QVariantMap resetReplay();

private:
    QString digest(const QStringList& parts) const;
    QString memberRoot() const;

    QSet<QString> executedActions_;
};
