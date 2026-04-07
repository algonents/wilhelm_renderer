#include <iostream>
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <ft2build.h>
#include FT_FREETYPE_H

extern "C"
{
    // GFLW
    GLFWwindow *_glfwCreateWindow(const char *title, int width, int height, GLFWframebuffersizefun callback);
    void _glfwSetWindowUserPointer(GLFWwindow *window, void *pointer);
    void *_glfwGetWindowUserPointer(GLFWwindow *window);

    void _glfwGetWindowContentScale(GLFWwindow *window, float* xscale, float* yscale);


    bool _glfwWindowShouldClose(GLFWwindow *window);
    void _glfwDestroyWindow(GLFWwindow *window);
    void _glfwTerminate();

    void _glfwSwapBuffers(GLFWwindow *window);
    void _glfwPollEvents();

    double _glfwGetTime();

    void _glfwSetWindowSizeCallback(GLFWwindow *window, GLFWwindowsizefun callback);
    void _glfwSetScrollCallback(GLFWwindow *window, GLFWscrollfun callback);
    void _glfwSetCursorPosCallback(GLFWwindow *window, GLFWcursorposfun callback);
    void _glfwSetKeyCallback(GLFWwindow *window, GLFWkeyfun callback);

    void _glfwGetWindowSize(GLFWwindow *window, int *width, int *height);
    void _glfwWindowHint(int hint, int value);

    // GL
    void _glClearColor(GLfloat x, GLfloat y, GLfloat z, GLfloat a);
    void _glViewPort(GLint x, GLint y, GLsizei width, GLsizei height);
    void _glGetIntegerv(GLenum pname, GLint *data);

    GLuint _glCreateShader(GLenum shaderType);
    void _glShaderSource(GLuint shader, GLchar *source);
    void _glCompileShader(GLuint shader);
    void _glDeleteShader(GLuint shader);
    void _glGetShaderiv(GLuint shader, GLenum pname, GLint *params);
    GLuint _glCreateProgram();
    void _glAttachShader(GLuint program, GLuint shader);
    void _glLinkProgram(GLuint program);
    void _glDeleteProgram(GLuint program);
    void _glUseProgram(GLuint program);
    GLuint _glGenBuffer();
    void _glGenBuffers(GLsizei n, GLuint *buffers);
    void _glBindBuffer(GLenum target, GLuint buffer);
    void _glBufferData(GLenum mode, GLsizeiptr size, const GLvoid *data, GLenum usage);
    void _glBufferSubData(GLenum target, GLintptr offset, GLsizeiptr size, const GLvoid *data);
    void _glDeleteBuffer(GLuint buffer);

    void _glActiveTexture(GLenum unit);
    GLint _glGenTexture();
    void _glBindTexture(GLenum target, GLuint texture);
    void _glTexImage2D(GLenum target, GLint level, GLint internalformat, GLsizei width, GLsizei height, GLint border, GLenum format, GLenum type, const void *data);
    void _glTexSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset, GLsizei width, GLsizei height, GLenum format, GLenum type, const void *data);
    void _glTexParameteri(GLenum target, GLenum pname, GLint param);
    void _glGenerateMipmap(GLenum target);
    void _glPixelStorei(GLenum pname, GLint param);
    void _glDeleteTexture(GLuint texture);

    GLuint _glGenVertexArray();
    void _glDeleteVertexArray(GLuint vao);
    void _glBindVertexArray(GLuint VAO);
    void _glVertexAttribPointer(GLuint index, GLint size, GLenum type, GLboolean normalized, GLsizei stride, GLsizei offset);
    void _glEnableVertexAttribArray(GLuint index);

    void _glDrawArrays(GLenum mode, GLint first, GLsizei count);
    void _glDrawArraysInstanced(GLenum mode, GLint first, GLsizei count, GLsizei instancecount);
    void _glVertexAttribDivisor(GLuint index, GLuint divisor);

    void _glDrawElements(GLenum mode, GLsizei count, GLenum type, GLuint offset);
    GLint _glGetUniformLocation(GLuint program, GLchar *name);
    void _glUniform1f(GLint location, GLfloat v0);
    void _glUniform2f(GLint location, GLfloat v0, GLfloat v1);
    void _glUniform3f(GLint location, GLfloat v0, GLfloat v1, GLfloat v2);
    void _glUniform4f(GLint location, GLfloat v0, GLfloat v1, GLfloat v2, GLfloat v3);
    void _glUniformMatrix4fv(GLint location, GLsizei count, GLboolean transpose, const GLfloat *value);
    void _glPointSize(GLfloat size);
    void _glEnable(GLenum cap);
    void _glBlendFunc(GLenum sfactor, GLenum dfactor);

    // FreeType
    int _ft_init_freetype(FT_Library *library);
    void _ft_done_freetype(FT_Library library);

    int _ft_new_face(FT_Library library, const char *filepath, long face_index, FT_Face *face);
    void _ft_done_face(FT_Face face);
    int _ft_set_pixel_sizes(FT_Face face, unsigned int width, unsigned int height);
    int _ft_load_char(FT_Face face, unsigned long char_code, int load_flags);

    // Glyph access - returns pointers to data inside FT_Face->glyph
    struct FT_GlyphMetrics {
        int width;          // Glyph width in pixels
        int height;         // Glyph height in pixels (rows)
        int bearing_x;      // Horizontal bearing (left)
        int bearing_y;      // Vertical bearing (top)
        long advance;       // Horizontal advance (in 1/64th pixels)
    };

    void _ft_get_glyph_metrics(FT_Face face, FT_GlyphMetrics *metrics);
    unsigned char *_ft_get_glyph_bitmap(FT_Face face);
    int _ft_get_glyph_bitmap_pitch(FT_Face face);
};