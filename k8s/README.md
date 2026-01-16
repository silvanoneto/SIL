# ☸️ Kubernetes Deployment

Guia para deployment do em clusters Kubernetes edge.

## 🎯 Plataformas Suportadas

- **K3s** - Lightweight Kubernetes para edge
- **MicroK8s** - Kubernetes para workstations e edge
- **KubeEdge** - Kubernetes para edge computing
- **K0s** - Zero-friction Kubernetes
- **Kubernetes vanilla** (1.24+)

## 📋 Pré-requisitos

### Cluster Kubernetes

```bash
# Verificar cluster
kubectl version
kubectl get nodes

# Para K3s (Raspberry Pi)
curl -sfL https://get.k3s.io | sh -

# Para MicroK8s
snap install microk8s --classic
microk8s enable dns storage
```

### Ferramentas

```bash
kubectl --version      # >= 1.24
helm version          # >= 3.10 (opcional)
```

## 🚀 Quick Start

### 1. Deploy Básico

```bash
# Aplicar manifest
kubectl apply -f k8s/deployment.yaml

# Verificar deployment
kubectl get all -n -system

# Ver logs
kubectl logs -n -system -l app= -f
```

### 2. Preparar Volumes

```bash
# Criar diretório de programas no node
ssh user@edge-node "sudo mkdir -p /opt//programs /opt//data"

# Copiar programas
scp programs/*.lis user@edge-node:/opt//programs/

# Criar PV local (se não tiver storage class)
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolume
metadata:
  name: -programs-pv
spec:
  capacity:
    storage: 1Gi
  accessModes:
    - ReadOnlyMany
  hostPath:
    path: /opt//programs
    type: Directory
  storageClassName: local-path
EOF
```

### 3. Verificar Status

```bash
# Status dos pods
kubectl get pods -n -system -o wide

# Logs de um pod específico
kubectl logs -n -system -runtime-xxxxx-yyyyy

# Descrição detalhada
kubectl describe pod -n -system -runtime-xxxxx-yyyyy

# Events
kubectl get events -n -system --sort-by='.lastTimestamp'
```

## 🏗️ Componentes

### 1. Deployment (-runtime)

Deployment padrão com 3 réplicas:

```yaml
replicas: 3
resources:
  requests:
    cpu: "100m"
    memory: "128Mi"
  limits:
    cpu: "500m"
    memory: "512Mi"
```

**Escalar:**

```bash
# Manual
kubectl scale deployment/-runtime -n -system --replicas=5

# Auto-scaling (HPA configurado para 1-10 réplicas)
kubectl get hpa -n -system
```

### 2. DaemonSet (-node-agent)

Deploy em todos os edge nodes:

```bash
# Marcar nodes como edge
kubectl label nodes <node-name> node-role.kubernetes.io/edge=true

# Verificar DaemonSet
kubectl get ds -n -system
```

### 3. Service (Headless)

Service headless para P2P mesh:

```bash
# Resolver DNS dos pods
kubectl run -it --rm debug --image=busybox --restart=Never -- \
  nslookup -service.-system.svc.cluster.local
```

## 🔧 Configuração

### ConfigMap

Editar configuração:

```bash
kubectl edit configmap/-config -n -system

# Ou aplicar novo configmap
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: -config
  namespace: -system
data:
  RUST_LOG: "debug"
  SIL_P2P_PORT: "9999"
EOF

# Restart pods para aplicar
kubectl rollout restart deployment/-runtime -n -system
```

### Secrets

Para dados sensíveis:

```bash
# Criar secret
kubectl create secret generic -secrets \
  -n -system \
  --from-literal=api-key=your-secret-key

# Usar no deployment
# spec.containers.env:
# - name: API_KEY
#   valueFrom:
#     secretKeyRef:
#       name: -secrets
#       key: api-key
```

## 📊 Monitoring

### Logs

```bash
# Todos os pods
kubectl logs -n -system -l app= --tail=100 -f

# Pod específico
kubectl logs -n -system -runtime-xxxxx --follow

# Container específico em pod multi-container
kubectl logs -n -system pod-name -c container-name

# Logs anteriores (após crash)
kubectl logs -n -system -runtime-xxxxx --previous
```

### Metrics

```bash
# Recursos dos pods
kubectl top pods -n -system

# Recursos dos nodes
kubectl top nodes

# Descrição detalhada
kubectl describe pod -n -system -runtime-xxxxx
```

### Prometheus (Opcional)

Se tiver Prometheus Operator:

```bash
# Verificar ServiceMonitor
kubectl get servicemonitor -n -system

# Queries úteis
# - rate(container_cpu_usage_seconds_total{namespace="-system"}[5m])
# - container_memory_usage_bytes{namespace="-system"}
```

## 🔄 Updates

### Rolling Update

```bash
# Atualizar imagem
kubectl set image deployment/-runtime \
  -n -system \
  =:v2.0

# Verificar rollout
kubectl rollout status deployment/-runtime -n -system

# Histórico
kubectl rollout history deployment/-runtime -n -system

# Rollback
kubectl rollout undo deployment/-runtime -n -system
```

### Blue-Green Deployment

```bash
# Criar deployment "green"
kubectl apply -f k8s/deployment-green.yaml

# Testar green
kubectl port-forward -n -system svc/-service-green 8888:8888

# Trocar service
kubectl patch service -service -n -system \
  -p '{"spec":{"selector":{"version":"green"}}}'

# Remover deployment "blue" antigo
kubectl delete deployment -runtime-blue -n -system
```

## 🐛 Troubleshooting

### Pod não inicia

```bash
# Verificar events
kubectl describe pod -n -system -runtime-xxxxx

# Verificar logs
kubectl logs -n -system -runtime-xxxxx

# Verificar image pull
kubectl get events -n -system | grep -i pull

# Solução comum: ImagePullPolicy
# spec.containers.imagePullPolicy: IfNotPresent
```

### Out of Memory

```bash
# Aumentar limits
kubectl patch deployment -runtime -n -system \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"","resources":{"limits":{"memory":"1Gi"}}}]}}}}'

# Verificar swap no node (K3s)
ssh user@node "sudo swapon --show"
```

### Node Affinity

```bash
# Verificar labels dos nodes
kubectl get nodes --show-labels

# Adicionar label
kubectl label nodes node1 node-role.kubernetes.io/edge=true

# Remover label
kubectl label nodes node1 node-role.kubernetes.io/edge-
```

### PVC não monta

```bash
# Verificar PVC status
kubectl get pvc -n -system

# Verificar PV
kubectl get pv

# Logs do provisioner (se usar dynamic provisioning)
kubectl logs -n kube-system -l app=local-path-provisioner

# Solução para hostPath (K3s)
# Criar diretório manualmente no node
ssh user@node "sudo mkdir -p /opt/local-path-provisioner/-programs-pvc"
```

## 🔐 Segurança

### RBAC

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: -sa
  namespace: -system
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: -role
  namespace: -system
rules:
  - apiGroups: [""]
    resources: ["pods", "services"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: -rolebinding
  namespace: -system
subjects:
  - kind: ServiceAccount
    name: -sa
roleRef:
  kind: Role
  name: -role
  apiGroup: rbac.authorization.k8s.io
```

Aplicar:

```bash
kubectl apply -f k8s/rbac.yaml

# Usar no deployment:
# spec.serviceAccountName: -sa
```

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: -network-policy
  namespace: -system
spec:
  podSelector:
    matchLabels:
      app: 
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: 
      ports:
        - protocol: TCP
          port: 8888
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: 
      ports:
        - protocol: TCP
          port: 8888
```

### Pod Security Standards

```bash
# Aplicar Pod Security Standard
kubectl label namespace -system \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/warn=restricted
```

## 📦 Helm Chart (Futuro)

Estrutura proposta:

```
-chart/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── pvc.yaml
│   └── hpa.yaml
└── README.md
```

## 🌐 Multi-Cluster

Para deployment em múltiplos clusters edge:

```bash
# Usando kubefed
kubefedctl join cluster1 --cluster-context=cluster1 --host-cluster-context=host
kubefedctl join cluster2 --cluster-context=cluster2 --host-cluster-context=host

# Deploy federado
kubectl apply -f k8s/federated-deployment.yaml
```

## 📚 Exemplos

### Edge IoT Deployment

```bash
# 1. Deploy em Raspberry Pi cluster
kubectl apply -f k8s/deployment.yaml

# 2. Configurar node labels
kubectl label nodes rpi-01 hardware=raspberry-pi
kubectl label nodes rpi-01 sensor-type=camera,temperature

# 3. Deploy com node selector
kubectl patch deployment -runtime -n -system \
  -p '{"spec":{"template":{"spec":{"nodeSelector":{"hardware":"raspberry-pi"}}}}}'
```

### Swarm Intelligence

```bash
# Deploy múltiplas réplicas para swarm behavior
kubectl scale deployment/-runtime -n -system --replicas=10

# Verificar mesh P2P
kubectl exec -it -n -system -runtime-xxxxx -- \
  lis run /app/examples/swarm_test.lis
```

## 🌐 LIS API Server Deployment

Deploy do servidor REST API para compilação e execução de código LIS:

### Quick Start

```bash
# Deploy API server
kubectl apply -f k8s/api-deployment.yaml

# Verificar deployment
kubectl get all -n lis-api

# Ver logs
kubectl logs -n lis-api -l app=lis,component=api -f
```

### Acessar API

```bash
# Port forward para teste local
kubectl port-forward -n lis-api svc/lis-api 3000:80

# Testar endpoint
curl http://localhost:3000/health

# Compilar código
curl -X POST http://localhost:3000/api/compile \
  -H "Content-Type: application/json" \
  -d '{"source": "fn main() { return 42; }"}'

# Acessar Swagger UI
open http://localhost:3000/docs
```

### Configuração

```bash
# Editar ConfigMap
kubectl edit configmap/lis-api-config -n lis-api

# Habilitar autenticação (editar secret)
kubectl edit secret/lis-api-secrets -n lis-api

# Restart para aplicar
kubectl rollout restart deployment/lis-api -n lis-api
```

### Escalar

```bash
# Manual
kubectl scale deployment/lis-api -n lis-api --replicas=5

# HPA configurado para 2-20 réplicas baseado em CPU/memória
kubectl get hpa -n lis-api
```

### Ingress

Configure o hostname no `api-deployment.yaml`:

```yaml
spec:
  rules:
    - host: lis-api.your-domain.com  # Altere aqui
```

Para TLS, descomente a seção `tls` e crie o secret:

```bash
kubectl create secret tls lis-api-tls \
  -n lis-api \
  --cert=path/to/cert.pem \
  --key=path/to/key.pem
```

## 🔗 Referências

- [deployment.yaml](deployment.yaml) - Runtime manifest
- [api-deployment.yaml](api-deployment.yaml) - API server manifest
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [K3s Documentation](https://docs.k3s.io/)
- [KubeEdge Documentation](https://kubeedge.io/docs/)

---

**⧑** *Kubernetes na borda — orquestração descentralizada.*
