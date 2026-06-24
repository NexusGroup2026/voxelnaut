# VoxelNaut

**VoxelNaut** é um jogo sandbox voxel completo em Rust, inspirado em mecânicas de exploração, construção, mineração, sobrevivência, crafting, viagem dimensional e muito mais.

⚠️ **Nota Importante sobre Assets e Design**

Este projeto é **100% original**:
- **Código**: Todo o código Rust é original e escrito do zero
- **Design do Dimensional Rift Engine**: Design ORIGINAL que não é baseado em nenhuma arma ou dispositivo de nenhum jogo existente (incluindo Portal da Valve). É um dispositivo de viagem dimensional criado especificamente para este projeto.
- **Sons**: Gerados proceduralmente ou são placeholders (sons do Minecraft são copyright Mojang/Microsoft e não são usados)
- **Texturas**: Geradas proceduralmente via código

---

## 🎮 Features Implementadas

### ✅ Sistema de Mundo Infinito
- Geração procedural de terreno com Perlin/Simplex noise
- **Mundo subterrâneo infinito**: Cavernas 3D, ravinas, minérios em diferentes profundidades
- Geração lazy: só gera chunks visíveis, nunca tudo de uma vez
- Descarrega chunks distantes automaticamente (memory management)

### ✅ Sistema de Dimensões (14 dimensões jogáveis)
- **Overworld**: Mundo principal
- **Lua**: Baixa gravidade, crateras
- **Marte**: Vermelho, poeiras, vulcões
- **Vênus**: Quente, atmosfera ácida
- **Mercúrio**: Extremas temperaturas
- **Júpiter**: Gigante gasoso com tempestades
- **Saturno**: Com anéis
- **Netuno**: Gigante de gelo
- **Plutão**: Mundo gelado
- **Cinturão de Asteroides**: Sem gravidade
- **The Void**: Dimensão vazia
- **Crystal Realm**: Dimensão de cristais
- **Ember Realm**: Dimensão de fogo
- **Frost Realm**: Dimensão de gelo

### ✅ Sistema de Viagem Dimensional

#### Dimensional Rift Engine (Original)
Dispositivo original para viajar entre dimensões.

**Receita de Crafting (3x3):**
```
[Iron] [Iron] [Iron]       -- Linha de cima: Ferro
[Gold] [Diamond] [Gold]    -- Linha do meio: Ouro - Diamante - Ouro
[Iron] [Iron] [Iron]       -- Linha de baixo: Ferro
```
Resultado: **1x Dimensional Rift Engine** (1000 usos)

#### Sistema de Cristais
Cada dimensão requer um cristal específico para ser acessada:
- Lua: Moon Crystal (220)
- Marte: Mars Crystal (221)
- Vênus: Venus Crystal (222)
- etc.

### ✅ Sistema de UI (egui)
- Menu principal com Singleplayer/Multiplayer/Settings
- HUD com vida, fome, hotbar, crosshair
- Tela de inventário com drag-and-drop
- Settings completos (Graphics, Audio, Controls)
- Debug info (F3)

### ✅ Sistema de Áudio (Rodio)
- 40+ sons registrados
- Sistema de música com crossfade
- Categorias: Master, Music, SFX, Ambient
- Soundscape procedural

### ✅ Sistema de Crafting
- Crafting 3x3
- 50+ receitas implementadas
- Receitas de ferramentas, blocos, e items especiais

### ✅ Sistema de Sobrevivência
- HP (vida)
- Fome e saturação
- Stamina
- Dano de queda
- Dano de afogamento
- Regeneração

### ✅ Sistema de Física
- Colisão AABB
- Raycasting para seleção de blocos
- Gravidade por dimensão (lua = 0.16x, Marte = 0.38x, etc.)
- Movimento com inércia

### ✅ Sistema de Iluminação
- Sky light (luz do sol)
- Block light (luz de blocos como tochas)
- Fog com distância

### ✅ Sistema de Chunks
- Chunks 32x32x256
- Lazy loading
- Frustum culling
- Multi-threaded generation

### ✅ Sistema P2P (Multiplayer)
- Conexão direta e STUN
- Sync de chunks
- Sync de entidades
- Anti-cheat

---

## 🏗️ Arquitetura

```
voxelnaut/
├── core/           # Tipos fundamentais, blocos, itens, entidades
├── world/          # Geração procedural, chunks, biomas, dimensões
├── render/         # Renderer WGPU, shaders, câmera
├── physics/        # Física, colisão, movimento
├── gameplay/       # Inventário, crafting, sobrevivência, viagem dimensional
├── ai/             # Pathfinding A*, comportamento de mobs
├── net/            # P2P, sincronização, anti-cheat
├── ui/             # Menu, HUD, inventário, settings
├── audio/          # Sistema de som
├── assets/         # Placeholder para assets
├── tools/          # Ferramentas de desenvolvimento
└── launcher/       # Entry point, game loop
```

---

## 🚀 Como Compilar e Jogar

### Pré-requisitos
- Rust 1.70+
- Git

### Build

```bash
# Clone o repositório
cd C:\Users\moises\voxelnaut

# Build em modo release
cargo build --release

# Ou use o script PowerShell
.\scripts\build.ps1 -Release
```

### Executar

```bash
# Modo release
cargo run --release

# Modo debug
cargo run
```

---

## 🎯 Como Usar o Dimensional Rift Engine

1. **Obtenha os materiais:**
   - 8 Iron Ingots (lingotes de ferro)
   - 2 Gold Ingots (lingotes de ouro)
   - 1 Diamond (diamante)

2. **Crafte o Dimensional Rift Engine:**
   - Abra a mesa de crafting 3x3
   - Coloque: Ferro-Ferro-Ferro / Ouro-Diamante-Ouro / Ferro-Ferro-Ferro
   - Receba 1x Dimensional Rift Engine

3. **Equipe o dispositivo:**
   - Coloque na mão (slot de ferramentas)

4. **Selecione o destino:**
   - Use scroll do mouse ou tecla para ciclhar entre dimensões
   - Ou use a roda de seleção no inventário

5. **Viaje:**
   - Segure clique direito para ativar o teletransporte
   - Você será movido para a nova dimensão

---

## ⚠️ Notas Importantes

1. **Este NÃO é Minecraft**: VoxelNaut é um projeto original inspirado em jogos voxel, não uma cópia ou clone.

2. **Dimensional Rift Engine**: Este é um dispositivo de viagem dimensional **100% original** criado especificamente para VoxelNaut. NÃO é baseado no Portal Gun da Valve, NÃO tem o mesmo design, e NÃO é uma cópia de nenhum dispositivo de nenhum jogo.

3. **Assets**: Este projeto não usa nenhum asset do Minecraft (sons, texturas, modelos). Todos os assets são gerados proceduralmente ou são placeholders.

4. **Estado do Projeto**: Este é um projeto em desenvolvimento ativo. Nem todas as features estão 100% implementadas.

---

## 📋 Roadmap

- [x] Sistema de chunks e mundo procedural
- [x] Sistema de renderização WGPU
- [x] Sistema de física e colisão
- [x] Inventário e crafting
- [x] Sistema de sobrevivência (HP, fome)
- [x] Sistema de dimensões (14 dimensões)
- [x] Dimensional Rift Engine (crafting e uso)
- [x] UI com egui (menu, HUD, inventário)
- [x] Sistema de áudio
- [x] Mundo subterrâneo infinito
- [ ] Implementação completa de todos os biomas dimensionais
- [ ] Renderização de água e lava
- [ ] Sistema de mobs completo
- [ ] Multiplayer funcional
- [ ] Persistência de mundo

---

## 📄 Licença

Este projeto é código aberto e livre para uso educacional e pessoal.
Não use assets do Minecraft (sons, texturas) com este projeto.
