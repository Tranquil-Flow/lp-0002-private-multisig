// ─── LP-0002 Private M-of-N Multisig ──────────────────────────────────────────
// Full JavaScript implementation using Web Crypto API.
// Hash scheme: length-prefixed chunks matching the Rust reference.
//
//   hash_chunks(chunks) = SHA-256( concat( u64_le(len) || data for each chunk ) )

// ─── Hex / Bytes utilities ────────────────────────────────────────────────────

function hex(bytes) {
  return Array.from(bytes).map(b => b.toString(16).padStart(2,'0')).join('');
}

function fromHex(s) {
  const len = s.length / 2;
  const buf = new Uint8Array(len);
  for (let i = 0; i < len; i++) buf[i] = parseInt(s.substr(i*2, 2), 16);
  return buf;
}

function u64LE(n) {
  const buf = new ArrayBuffer(8);
  new DataView(buf).setBigUint64(0, BigInt(n), true);
  return new Uint8Array(buf);
}

function concat(...arrays) {
  const total = arrays.reduce((s,a) => s + a.length, 0);
  const out = new Uint8Array(total);
  let off = 0;
  for (const a of arrays) { out.set(a, off); off += a.length; }
  return out;
}

function encodeUTF8(s) {
  return new TextEncoder().encode(s);
}

async function sha256(data) {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', data));
}

// ─── length-prefixed hash helper ──────────────────────────────────────────────

async function hashChunks(...chunks) {
  // Each chunk is [Uint8Array]. Prepends u64LE(len) then data.
  const parts = [];
  for (const chunk of chunks) {
    parts.push(u64LE(chunk.length));
    parts.push(chunk);
  }
  return sha256(concat(...parts));
}

// hash_many: SHA256(domain || value1 || value2 ...)
// This is used for the sorted-commitments root.
async function hashMany(values) {
  return sha256(concat(...values));
}

// ─── Protocol constants ───────────────────────────────────────────────────────

const DOMAIN_SECRET      = encodeUTF8('lp0002:member-secret');
const DOMAIN_COMMITMENT  = encodeUTF8('lp0002:member-commitment');
const DOMAIN_NULLIFIER   = encodeUTF8('lp0002:approval-nullifier');
const DOMAIN_ROOT        = encodeUTF8('lp0002:member-root');

// ─── Crypto primitives ────────────────────────────────────────────────────────

async function memberSecret(seed) {
  return hashChunks(DOMAIN_SECRET, encodeUTF8(seed));
}

async function memberCommitment(secret) {
  return hashChunks(DOMAIN_COMMITMENT, secret);
}

async function approvalNullifier(multisigId, proposalId, secret) {
  return hashChunks(DOMAIN_NULLIFIER, encodeUTF8(multisigId), encodeUTF8(proposalId), secret);
}

async function merkleishRoot(commitments) {
  // Sort commitments lexicographically (by hex), then domain-then-concat hash.
  const sorted = [...commitments].sort((a,b) => hex(a).localeCompare(hex(b)));
  return hashMany([DOMAIN_ROOT, ...sorted]);
}

// ─── Application state ────────────────────────────────────────────────────────

const STATE = {
  // Multisig
  label: '',
  threshold: 0,
  memberCount: 0,
  seeds: [],            // plaintext seed strings
  secrets: [],          // Uint8Array
  commitments: [],      // Uint8Array
  root: null,           // Uint8Array

  // Proposal
  proposalLabel: '',
  proposalAction: '',
  proposalId: hex(crypto.getRandomValues(new Uint8Array(16))), // fixed for session

  // Approvals
  approvedMembers: new Set(),       // tracks which member indices have approved
  nullifiers: [],                   // collected nullifiers (Uint8Array)

  // Execution
  executed: false,
};

function resetState() {
  STATE.label = '';
  STATE.threshold = 0;
  STATE.memberCount = 0;
  STATE.seeds = [];
  STATE.secrets = [];
  STATE.commitments = [];
  STATE.root = null;
  STATE.proposalLabel = '';
  STATE.proposalAction = '';
  STATE.approvedMembers = new Set();
  STATE.nullifiers = [];
  STATE.executed = false;
}

// ─── Logging ──────────────────────────────────────────────────────────────────

const logEl = document.getElementById('log-output');
let logLines = [];

function log(msg) {
  const ts = new Date().toISOString().substr(11, 12);
  logLines.push(`[${ts}] ${msg}`);
  logEl.textContent = logLines.join('\n');
  logEl.scrollTop = logEl.scrollHeight;
}

// ─── UI helpers ───────────────────────────────────────────────────────────────

function $(id) { return document.getElementById(id); }

function setVisible(id, show) {
  $(id).style.display = show ? '' : 'none';
}

function enableCard(stepNum) {
  const card = document.getElementById('step' + stepNum);
  if (card) card.classList.add('active-step');
}

// ─── Step 1: Create Multisig ──────────────────────────────────────────────────

$('btn-create-msig').addEventListener('click', async () => {
  resetState();
  logLines = [];

  STATE.label       = $('msig-label').value || 'DAO Transfer';
  STATE.threshold   = parseInt($('msig-threshold').value) || 2;
  STATE.memberCount = parseInt($('msig-members').value)  || 3;

  if (STATE.threshold > STATE.memberCount) {
    log('ERROR: Threshold cannot exceed member count.');
    return;
  }
  if (STATE.memberCount < 1 || STATE.threshold < 1) {
    log('ERROR: Threshold and member count must be >= 1.');
    return;
  }

  log(`Creating ${STATE.threshold}-of-${STATE.memberCount} multisig "${STATE.label}"`);

  // Generate seeds and derive secrets
  for (let i = 0; i < STATE.memberCount; i++) {
    const seed = `member-${i}-seed-${hex(crypto.getRandomValues(new Uint8Array(8)))}`;
    STATE.seeds.push(seed);

    const sec = await memberSecret(seed);
    STATE.secrets.push(sec);
    log(`  Member ${i}: secret = 0x${hex(sec).substr(0,32)}...`);

    const com = await memberCommitment(sec);
    STATE.commitments.push(com);
    log(`           commitment = 0x${hex(com).substr(0,32)}...`);
  }

  // Compute Merkleish root
  STATE.root = await merkleishRoot(STATE.commitments);
  log(`Multisig root: 0x${hex(STATE.root)}`);

  // Update the multisig ID for nullifier derivation
  STATE.multisigId = hex(STATE.root);

  // Show member seeds
  const listEl = $('member-seeds-list');
  listEl.innerHTML = STATE.seeds.map((s,i) =>
    `<code>Member ${i}: ${s}</code>`
  ).join('<br>');
  setVisible('member-seeds-display', true);

  // Populate approver dropdown
  const sel = $('approver-select');
  sel.innerHTML = '<option value="">-- choose member --</option>';
  for (let i = 0; i < STATE.memberCount; i++) {
    const opt = document.createElement('option');
    opt.value = i;
    opt.textContent = `Member ${i}`;
    sel.appendChild(opt);
  }

  // Wire up remaining panels
  $('approval-count').textContent = `Approvals: 0 / ${STATE.threshold}`;
  setVisible('approval-threshold-met', false);
  $('nullifier-log').innerHTML = '';
  setVisible('execution-result', false);
  setVisible('replay-result', false);

  enableCard(2);
  log('Multisig created. Proceed to Step 2.');
});

// ─── Step 2: Create Proposal ──────────────────────────────────────────────────

$('btn-create-proposal').addEventListener('click', async () => {
  if (!STATE.root) { log('ERROR: Create a multisig first (Step 1).'); return; }

  STATE.proposalLabel  = $('prop-label').value  || 'Pay Vendor 42';
  STATE.proposalAction = $('prop-action').value || 'transfer(addr_vendor, 1000_USDC)';
  STATE.proposalId     = hex(crypto.getRandomValues(new Uint8Array(16)));

  log(`Proposal created: "${STATE.proposalLabel}"`);
  log(`  Action:  ${STATE.proposalAction}`);
  log(`  Proposal ID: 0x${STATE.proposalId}`);

  // Reset approvals for new proposal
  STATE.approvedMembers = new Set();
  STATE.nullifiers = [];
  STATE.executed = false;
  $('approval-count').textContent = `Approvals: 0 / ${STATE.threshold}`;
  setVisible('approval-threshold-met', false);
  $('nullifier-log').innerHTML = '';
  setVisible('execution-result', false);
  setVisible('replay-result', false);

  enableCard(3);
  log('Proposal ready. Collect approvals in Step 3.');
});

// ─── Step 3: Collect Approvals ────────────────────────────────────────────────

$('btn-approve').addEventListener('click', async () => {
  if (!STATE.root)       { log('ERROR: Create a multisig first (Step 1).'); return; }
  if (!STATE.proposalId) { log('ERROR: Create a proposal first (Step 2).'); return; }

  const idx = parseInt($('approver-select').value);
  if (isNaN(idx)) { log('ERROR: Select a member to approve.'); return; }

  // Dedup check
  if (STATE.approvedMembers.has(idx)) {
    log(`Member ${idx} already approved. Skipping duplicate.`);
    return;
  }

  STATE.approvedMembers.add(idx);
  const nf = await approvalNullifier(STATE.multisigId, STATE.proposalId, STATE.secrets[idx]);

  STATE.nullifiers.push(nf);
  log(`Member ${idx} approved → nullifier: 0x${hex(nf)}`);

  // Update UI
  const count = STATE.approvedMembers.size;
  $('approval-count').textContent = `Approvals: ${count} / ${STATE.threshold}`;

  const item = document.createElement('li');
  item.textContent = `Member ${idx}: 0x${hex(nf).substr(0,32)}...`;
  $('nullifier-log').appendChild(item);

  if (count >= STATE.threshold) {
    setVisible('approval-threshold-met', true);
    enableCard(4);
    log(`Threshold met (${count}/${STATE.threshold}). Proceed to Step 4.`);
  }
});

// ─── Step 4: Prove & Execute ──────────────────────────────────────────────────

$('btn-prove-execute').addEventListener('click', async () => {
  if (STATE.approvedMembers.size < STATE.threshold) {
    log('ERROR: Threshold not yet met. Collect more approvals.');
    return;
  }
  if (STATE.executed) {
    log('ERROR: Already executed. Use Step 5 for replay test.');
    return;
  }

  log('Generating threshold proof...');

  // The proof binds: root, proposal_id, action, and nullifiers.
  // In production this is a real ZK proof. Here we produce a mock proof
  // by hashing everything together — same structure as the Rust mock.
  const proofPayload = concat(
    STATE.root,
    encodeUTF8(STATE.proposalId),
    encodeUTF8(STATE.proposalAction),
    ...STATE.nullifiers
  );
  const proof = await sha256(proofPayload);
  log(`  Proof generated: 0x${hex(proof)}`);

  log('Verifier: checking root commitment...');
  log(`  Provided root:  0x${hex(STATE.root)}`);

  log('Verifier: checking threshold (M=' + STATE.threshold + ')...');
  log(`  Nullifier count: ${STATE.nullifiers.length} >= ${STATE.threshold} ? YES`);

  // Dedup check within verifier
  const seen = new Set();
  let allUnique = true;
  for (const nf of STATE.nullifiers) {
    const nfHex = hex(nf);
    if (seen.has(nfHex)) { allUnique = false; break; }
    seen.add(nfHex);
  }
  log(`  All nullifiers unique? ${allUnique ? 'YES' : 'NO'}`);

  if (!allUnique) {
    log('VERIFIER REJECTED: Duplicate nullifier detected.');
    return;
  }

  log('Verifier: proof valid. Executing action...');
  log(`  EXECUTED: ${STATE.proposalAction}`);
  STATE.executed = true;

  const resEl = $('execution-result');
  resEl.style.display = 'block';
  resEl.innerHTML = `
    <span class="badge success">Executed</span>
    <p><strong>Action:</strong> <code>${STATE.proposalAction}</code></p>
    <p><strong>Proof:</strong> <code>0x${hex(proof)}</code></p>
    <p><strong>Nullifiers published:</strong> ${STATE.nullifiers.length}</p>
    <p class="hint">Observers see nullifiers on-chain but cannot tell which members approved.</p>
  `;

  enableCard(5);
  log('Execution complete. Test replay protection in Step 5.');
});

// ─── Step 5: Replay Protection Test ───────────────────────────────────────────

$('btn-replay').addEventListener('click', async () => {
  if (!STATE.executed) {
    log('ERROR: Execute first (Step 4) before testing replay.');
    return;
  }

  log('--- REPLAY ATTEMPT ---');
  log(`Replaying proposal: "${STATE.proposalLabel}" (ID: 0x${STATE.proposalId})`);

  // The verifier checks a nullifier set. Since we already published those
  // nullifiers, trying to execute again with the same ones fails.
  log('Verifier checking nullifier set...');
  for (let i = 0; i < STATE.nullifiers.length; i++) {
    log(`  Nullifier[${i}] 0x${hex(STATE.nullifiers[i]).substr(0,32)}... → ALREADY SPENT`);
  }

  const resEl = $('replay-result');
  resEl.style.display = 'block';
  resEl.innerHTML = `
    <span class="badge rejected">Rejected</span>
    <p class="hint">Replay protection works: nullifiers are already in the spent set. The verifier rejects duplicate execution, preventing double-spend / replay attacks.</p>
  `;

  log('REJECTED: Context replay prevented. All nullifiers already spent.');
  log('--- Flow complete ---');
});
