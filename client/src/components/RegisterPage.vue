<script setup>
import { ref } from 'vue';
import ProfileThumbnail from './ProfileThumbnail.vue';

const profilePictureSource = ref('./src/assets/profile-pictures/default-profile.png');

function onUserProfileImageSelected(_) {
  const [file] = document.getElementById('profile').files
  if (file) {
    profilePictureSource.value = URL.createObjectURL(file)
  }
}
</script>

<template>
  <div class="flex flex-col gap-y-8 justify-center items-center bg-gray-950 h-screen">
    <h1 class="text-gray-100 text-4xl font-bold">Register</h1>

    <form class="flex flex-col gap-y-2 w-[50%]" action="https://localhost:8443/api/user/register" method="post"
      enctype="multipart/form-data">
      <img class="w-48 h-48 self-center" :src="profilePictureSource" alt="Profile picture">

      <input class="rounded-lg p-2 bg-gray-800 text-gray-100" type="text" name="username" id="username"
        placeholder="Username" required>
      <input class="rounded-lg p-2 bg-gray-800 text-gray-100" type="password" name="password" id="password"
        placeholder="Password" required>
      <input class="rounded-lg p-2 bg-gray-800 text-gray-100" type="text" name="nickname" id="nickname"
        placeholder="Nickname (visible to others)" required>
      <input class="self-center" type="file" name="profile" id="profile" accept="image/*"
        @change="onUserProfileImageSelected">

    <input class="bg-blue-600 text-gray-200 font-bold px-4 py-2 rounded-lg" type="submit" value="Register">
  </form>
</div></template>