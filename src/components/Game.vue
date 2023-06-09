<template>
  <Transition>
    <div v-if="alert" class="alert">{{ alert }}</div>
  </Transition>
  <div class="game">
    <button @click="restartGame" class="reset">
      <img src="../assets/reset.svg" class="reset-icon" />
    </button>
    <div class="table">
      <Piles :piles="piles" :pickPile="pickPile" />
      <div class="info">
        <div class="round">
          <div>Round: {{ round }}</div>
          <div class="scores">
            <template v-for="(player, i) in playersSorted" :key="i">
              <div :class="{ me: player.me }">{{ player.emoji }} {{ player.online }} {{ player.name }}: {{ player.points
              }}
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
    <CardHand :cards="hand" :playedCard="playedCard" :sendPlayCard="sendPlayCard" />
  </div>
</template>

<script>
import CardHand from './CardHand.vue'
import Piles from './Piles.vue'

export default {
  name: 'Game',
  components: {
    CardHand, Piles
  },
  props: {
    players: {
      type: Object,
      required: true,
    },
    round: {
      type: Number,
      required: true,
    },
    playedCard: {
      type: [Number, null],
      required: true,
    },
    piles: {
      type: Array,
      required: true,
    },
    hand: {
      type: Array,
      required: true,
    },
    pickPile: {
      type: [Function, null],
      required: true,
    },
    sendPlayCard: {
      type: Function,
      required: true,
    },
    restartGame: {
      type: Function,
      required: true,
    }
  },
  data() {
    return {
      alert: null
    }
  },
  computed: {
    playersSorted() {
      const players = Object.entries(this.players).map(([name, info]) => {
        let emoji = '⏳';
        if (info.played === "played") {
          emoji = '✅';
        } else if (info.played === 'must_pick_pile') {
          emoji = '⭕';
        }
        return {
          name,
          points: info.points,
          me: info.me,
          online: info.online ? '🟢' : '🔴',
          emoji
        }
      })
      players.sort((a, b) => {
        if (a.me) {
          return -Infinity;
        }
        b.points - a.points
      })
      return players
    }
  },
  watch: {
    players(newPlayers, oldPlayers) {
      const findMe = p => Object.values(p).find(p => p.me)
      const oldMe = findMe(oldPlayers);
      const newMe = findMe(newPlayers);
      if (oldMe.points !== newMe.points) {
        const points = newMe.points - oldMe.points;
        const s = points === 1 ? '' : 's';
        const alert = `You got ${points} point${s}!`;
        this.alert = alert;
        setTimeout(() => {
          if (alert === this.alert) {
            this.alert = null;
          }
        }, 3000);
      } else if (newMe.played === 'must_pick_pile') {
        this.alert = 'You must pick a pile!';
      }
    },
    round(newRound, oldRound) {
      if (newRound !== oldRound) {
        this.alert = 'New round!';
        setTimeout(() => {
          this.alert = null;
        }, 3000);
      }
    }
  },
}
</script>


<style scoped>
.game {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.table {
  width: 90%;
  display: flex;
  align-items: center;
}

.info {
  font-weight: bold;
  width: 20%;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
}

.round {
  background-color: #38A3A5;
  width: 80px;
  padding: 10px;
  margin: 10px;
}

.reset {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  margin: 10px;
  display: inline-block;
  cursor: pointer;
  border: none;
  background: #57CC99;
  color: #111;
  display: flex;
  justify-content: center;
  align-items: center;
  align-self: end;
  margin-bottom: -40px;
}

.reset-icon {
  height: 15px;
  width: 15px;
}

.alert {
  background-color: #f44336;
  color: white;
  font-weight: bold;
  padding: 20px;
  font-size: 2rem;
}

.v-enter-active,
.v-leave-active {
  transition: opacity 0.5s ease;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}

.scores {
  font-weight: normal;
}

.me {
  font-weight: bold;
}
</style>