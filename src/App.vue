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
        })
        .catch(error => {
          console.error('Error:', error);
        });

    }
  },
  mounted() {
    this.userId = localStorage.getItem("user_id");
    console.log("Retrieved the following user_id from local storage: ", this.userId);
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
