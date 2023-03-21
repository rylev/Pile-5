<template>
  <template v-if="!userId">
    <form @submit.prevent="handleSubmit">
      <label for="my-input">Enter your name:</label>
      <input type="text" id="my-input" v-model="playerName">
      <button type="submit">Submit</button>
    </form>
  </template>
  <NetworkedApp v-else :userId="userId" />
</template>

<script>
import NetworkedApp from './components/NetworkedApp.vue'

export default {
  name: 'App',
  components: {
    NetworkedApp
  },
  data() {
    return {
      userId: null,
      playerName: null
    }
  },

  methods: {
    handleSubmit() {
      console.log(`Request for ${this.playerName} to join the game`);
      fetch("/join", {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ name: this.playerName })
      })
        .then(response => {
          if (!response.ok) {
            throw new Error('Network response was not ok');
          }
          return response.json();
        })
        .then(json => {
          console.log('Success:', JSON.stringify(json));
          this.userId = json.user_id;
          localStorage.setItem("user_id", this.userId);
          localStorage.setItem("version", json.version);
        })
        .catch(error => {
          console.error('Error:', error);
        });

    }
  },
  mounted() {
    const version = localStorage.getItem("version");
    this.userId = localStorage.getItem("user_id");
    console.log("UserId: ", this.userId, " Version: ", version);
    if (version && this.userId) {
      fetch("/version").then(r => r.text()).then(v => {
        if (v !== version) {
          localStorage.clear();
          this.userId = null;
        }
      })
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
