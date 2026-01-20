module.exports = function(source) {
  return source.replace(/import\.meta\.url/g, 'window.location.href');
};
