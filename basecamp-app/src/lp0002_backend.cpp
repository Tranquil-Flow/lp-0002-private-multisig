#include "lp0002_backend.h"

#include <QCryptographicHash>
#include <QJsonDocument>
#include <QJsonObject>
#include <QtGlobal>

#include <iterator>

namespace {
struct Member {
    const char* label;
    const char* secret;
};

constexpr Member kMembers[] = {
    {"alice", "alice-shielded-key"},
    {"boris", "boris-shielded-key"},
    {"cyra", "cyra-shielded-key"},
    {"dara", "dara-shielded-key"},
    {"evan", "evan-shielded-key"},
};
}

Lp0002Backend::Lp0002Backend(QObject* parent)
    : QObject(parent)
{
}

QVariantMap Lp0002Backend::proveThreshold(int threshold,
                                          const QVariantList& approvals,
                                          const QString& proposalId,
                                          const QString& actionText)
{
    const int boundedThreshold = qBound(1, threshold, static_cast<int>(std::size(kMembers)));
    QVariantList nullifiers;
    QStringList nullifierParts;
    int approvalCount = 0;

    for (int i = 0; i < static_cast<int>(std::size(kMembers)); ++i) {
        const bool approved = i < approvals.size() && approvals.at(i).toBool();
        if (!approved) {
            continue;
        }

        ++approvalCount;
        const QString nullifier = digest({
            QStringLiteral("lp0002:approval-nullifier"),
            QStringLiteral("treasury-v1"),
            proposalId,
            QString::fromUtf8(kMembers[i].secret),
        });
        const QString compact = nullifier.left(32);
        nullifiers.append(compact);
        nullifierParts.append(compact);
    }

    const QString actionHash = digest({QStringLiteral("lp0002:action"), actionText});
    const bool thresholdMet = approvalCount >= boundedThreshold;
    const QString root = memberRoot();
    const QString receiptId = digest({
        QStringLiteral("lp0002:receipt"),
        QStringLiteral("treasury-v1"),
        root,
        proposalId,
        actionHash,
        QString::number(approvalCount),
        QString::number(boundedThreshold),
        nullifierParts.join(QStringLiteral("|")),
    });

    QVariantMap journal;
    journal.insert(QStringLiteral("multisig_id"), QStringLiteral("treasury-v1"));
    journal.insert(QStringLiteral("member_root"), root);
    journal.insert(QStringLiteral("proposal_id"), proposalId);
    journal.insert(QStringLiteral("action_hash"), actionHash);
    journal.insert(QStringLiteral("approval_count"), approvalCount);
    journal.insert(QStringLiteral("threshold"), boundedThreshold);
    journal.insert(QStringLiteral("nullifiers"), nullifiers);
    journal.insert(QStringLiteral("threshold_met"), thresholdMet);

    QVariantMap result;
    result.insert(QStringLiteral("ok"), thresholdMet);
    result.insert(QStringLiteral("status"), thresholdMet ? QStringLiteral("ThresholdMet") : QStringLiteral("InsufficientApprovals"));
    result.insert(QStringLiteral("privacy"), QStringLiteral("Public journal exposes counts, nullifiers, hashes, and root; signer identities and shielded secrets stay local."));
    result.insert(QStringLiteral("receipt_id"), receiptId.left(40));
    result.insert(QStringLiteral("journal"), journal);
    return result;
}

QVariantMap Lp0002Backend::executeJournal(const QVariantMap& journal)
{
    QVariantMap result;
    if (!journal.value(QStringLiteral("threshold_met")).toBool()) {
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("status"), QStringLiteral("InsufficientApprovals"));
        result.insert(QStringLiteral("message"), QStringLiteral("Verifier rejected execution because the approval count is below threshold."));
        return result;
    }

    const QString proposalId = journal.value(QStringLiteral("proposal_id")).toString();
    const QString actionHash = journal.value(QStringLiteral("action_hash")).toString();
    const QString executionKey = digest({QStringLiteral("lp0002:execution-key"), proposalId, actionHash});

    if (executedActions_.contains(executionKey)) {
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("status"), QStringLiteral("ReplayRejected"));
        result.insert(QStringLiteral("message"), QStringLiteral("The same proposal/action pair has already been executed."));
        return result;
    }

    executedActions_.insert(executionKey);
    result.insert(QStringLiteral("ok"), true);
    result.insert(QStringLiteral("status"), QStringLiteral("ExecutedOnce"));
    result.insert(QStringLiteral("message"), QStringLiteral("LEZ wrapper accepted the threshold journal and marked the action executed."));
    result.insert(QStringLiteral("execution_id"), executionKey.left(40));
    return result;
}

QVariantMap Lp0002Backend::checkReplay(const QVariantMap& journal) const
{
    const QString executionKey = digest({
        QStringLiteral("lp0002:execution-key"),
        journal.value(QStringLiteral("proposal_id")).toString(),
        journal.value(QStringLiteral("action_hash")).toString(),
    });

    QVariantMap result;
    const bool replay = executedActions_.contains(executionKey);
    result.insert(QStringLiteral("ok"), !replay);
    result.insert(QStringLiteral("status"), replay ? QStringLiteral("ReplayWouldFail") : QStringLiteral("FreshAction"));
    result.insert(QStringLiteral("execution_id"), executionKey.left(40));
    return result;
}

QVariantMap Lp0002Backend::resetReplay()
{
    executedActions_.clear();
    QVariantMap result;
    result.insert(QStringLiteral("ok"), true);
    result.insert(QStringLiteral("status"), QStringLiteral("ReplayStateReset"));
    return result;
}

QString Lp0002Backend::digest(const QStringList& parts) const
{
    QCryptographicHash hash(QCryptographicHash::Sha256);
    for (const QString& part : parts) {
        hash.addData(part.toUtf8());
        hash.addData("\x1f", 1);
    }
    return QString::fromLatin1(hash.result().toHex());
}

QString Lp0002Backend::memberRoot() const
{
    QStringList commitments;
    commitments.reserve(static_cast<int>(std::size(kMembers)));
    for (const Member& member : kMembers) {
        commitments.append(digest({
            QStringLiteral("lp0002:member"),
            QString::fromUtf8(member.secret),
        }));
    }
    commitments.sort();
    return digest({QStringLiteral("lp0002:member-root"), commitments.join(QStringLiteral("|"))});
}
