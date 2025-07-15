# 🤖 Projecte de Crowdfunding per Mentors Digitals

Aquest projecte permet recaptar fons de manera descentralitzada, transparent i segura mitjançant un contracte intel·ligent a la blockchain de **MultiversX**. Està pensat per a mentors digitals organitzats per **zones territorials**, que poden recaptar fons per adquirir material de robòtica i compartir-lo entre centres.

## ⚙️ Característiques principals

- ✅ Inici de campanya amb:
  - `target` (objectiu de recaptació)
  - `deadline` (data límit)
  - `min_deposit` i `max_per_wallet`
  - `max_total_project = target + 10%`
  - Dos administradors: `admin1` (coordinador de zona) i `admin2` (coordinador territorial)

- ✅ Aportacions amb EGLD via `fund()`, validant:
  - Que la campanya no ha acabat (`deadline` ni `terminated`)
  - Que no se superen els límits mínim, per wallet i total

- ✅ Funcions d’administració:
  - `claimFunds()` → Si s’ha assolit l’objectiu, els admins poden retirar els fons
  - `terminateCampaign()` → Finalitza la campanya anticipadament
  - `uploadInvoiceHash()` → Per afegir una factura associada a la despesa

- ✅ Funció per a donants:
  - `claimRefund()` → Si no s’assoleix l’objectiu abans del `deadline` o es cancel·la, els donants poden recuperar els fons

- ✅ Consultes via `view`:
  - `getContractAddress()`, `isAdmin()`, etc.

## 📦 Desplegament

Per desplegar el contracte:
```bash
mxpy contract deploy \
  --bytecode output/crowdfunding.wasm \
  --pem path/to/admin1.pem \
  --gas-limit=60000000 \
  --arguments 100000000000000000000 target \
              1753910400 deadline_unix_timestamp \
              1000000000000000000 min_deposit \
              50000000000000000000 max_per_wallet \
              erd1...admin2_address \
  --recall-nonce --send --outfile deploy.json \
  --proxy https://devnet-gateway.multiversx.com \
  --chain D
