import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import * as VueRouter from 'vue-router'
import MainPage from './components/main-page/MainPage.vue'
import LoginPage from './components/LoginPage.vue'
import LogoutPage from './components/LogoutPage.vue'
import RegisterPage from './components/RegisterPage.vue'
import UpdateProfilePage from './components/UpdateProfilePage.vue'

// Vuetify
import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

const vuetify = createVuetify({
  components,
  directives,
})

async function getLoginInfo() {
  const response = await fetch('https://localhost:8443/api/user/login_info', { mode: 'cors', credentials: 'include' });
  if (response && response.status == 200) {
    return await response.json();
  }
  else {
    return null;
  }
}

const routes = [
  {
    path: '/', component: MainPage, beforeEnter: async (to, from) => {
      // Check if user is logged in.
      const is_logged_in = (await getLoginInfo()) != null;

      if (!is_logged_in) {
        return '/login'; // Redirect to login page.
      }
    }
  },
  {
    path: '/login', component: LoginPage, beforeEnter: async (to, from) => {
      // Check if user is logged in.
      const is_logged_in = (await getLoginInfo()) != null;

      if (is_logged_in) {
        alert('You are already logged in. If you want to log in with another account, please log out first.');
        return '/'; // Redirect to main page.
      }
    }
  },
  {
    path: '/logout', component: LogoutPage
  },
  {
    path: '/register', component: RegisterPage, beforeEnter: async (to, from) => {
      // Check if user is logged in.
      const is_logged_in = (await getLoginInfo()) != null;

      if (is_logged_in) {
        alert('You are already logged in. If you want to register a new account, please log out first.');
        return '/'; // Redirect to main page.
      }
    }
  },
  {
    path: '/update_profile', component: UpdateProfilePage, beforeEnter: async (to, from) => {
      // Check if user is logged in.
      const is_logged_in = (await getLoginInfo()) != null;

      if (!is_logged_in) {
        return '/login'; // Redirect to login page.
      }
    }
  }
];

const router = VueRouter.createRouter({
  // 4. Provide the history implementation to use. We are using the hash history for simplicity here.
  history: VueRouter.createWebHashHistory(),
  routes, // short for `routes: routes`
});

createApp(App).use(router).use(vuetify).mount('#app')
