<template>
  <Game v-if="state && state.round > 0" :state="state" />
  <Lobby v-else-if="state" :players="state.players" :readyToPlay="readyToPlay" />
</template>

<script>
import Game from './Game.vue'
import Lobby from './Lobby.vue'
import { reactive } from 'vue'

export default {
  name: 'NetworkedApp',
  components: {
    Game, Lobby
  },
  props: {
    userId: {
      type: String,
      required: true,
    },
  },
  created() {
    const socket = new WebSocket(`ws://${location.host}/ws?user_id=${this.userId}`)

    socket.onopen = () => {
      console.log('WebSocket connected')
    }

    socket.onmessage = (message) => {
      console.log('WebSocket message received:', message.data)
      this.state = JSON.parse(message.data);
    }

    socket.onclose = () => {
      console.log('WebSocket disconnected')
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
    }
  },
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
