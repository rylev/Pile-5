<template>
  <Game v-if="state && state.state == 'game'" :players="state.players" :round="state.round.number" :hand="state.hand"
    :piles="state.piles" :playedCard="state.round.played" :pickPile="pickPile" :sendPlayCard="sendPlayCard"
    :restartGame="startOver" />

  <Lobby v-else-if="state && state.state == 'lobby'" :players="state.players" :readyToPlay="readyToPlay" />
  <GameOver v-else-if="state && state.state === 'game_over'" :scores="state.players" :startOver="startOver" />
</template>

<script>
import Game from './Game.vue'
import Lobby from './Lobby.vue'
import GameOver from './GameOver.vue'
import { reactive } from 'vue'

export default {
  name: 'NetworkedApp',
  components: {
    Game, Lobby, GameOver
  },
  props: {
    userId: {
      type: String,
      required: true,
    },
    authenticationFailed: {
      type: Function,
      required: true,
    }
  },
  created() {
    const socket = new WebSocket(`ws://${location.host}/ws?user_id=${this.userId}`)

    socket.onerror = (error) => {
      console.log('WebSocket error:', error)
    }

    socket.onopen = () => {
      console.log('WebSocket connected')
    }

    socket.onmessage = (message) => {
      console.log('WebSocket message received:', message.data)
      this.state = JSON.parse(message.data);
    }

    socket.onclose = (e) => {
      if (e.code === 1008) {
        console.log('WebSocket closed with error:', e.reason)
        this.authenticationFailed();
      }
      console.log('WebSocket disconnected', e)
    }

    this.socket = reactive({
      instance: socket,
    })
  },
  data() {
    return {
      state: null
    }
  },
  methods: {
    sendMessage(message) {
      this.socket.instance.send(JSON.stringify(message))
    },
    readyToPlay() {
      this.sendMessage({ event: "start_game" })
    },
    sendPlayCard(card) {
      this.sendMessage({ event: "play_card", card })
    },
    startOver() {
      this.sendMessage({ event: "restart_game" })
    }
  },
  computed: {
    pickPile() {
      if (this.state.round.state === 'select_pile') {
        return (index) => {
          this.sendMessage({ event: "select_pile", pile_index: index })
        }
      } else {
        return null;
      }
    }
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
}
</style>
